import { ajax, AjaxRequest } from 'rxjs/ajax'
import { Observable, Subscription } from 'rxjs/index'

const DEFAULT_RETRIES = 3
const DEFAULT_DELAY = 500
const DEFAULT_DELAY_MAX = 30000
const DEFAULT_TIMEOUT = 1500
const DEFAULT_TIMEOUT_MAX = 30000

export interface RequestOptions extends AjaxRequest, RetryOptions {}

const DEFAULT_HEADERS = {
	'Content-Type': 'application/json',
}

const DEFAULT_OPTIONS = {}

/**
 * Executes an Ajax request with advanced retry and timeout options.
 */
export function request(options: RequestOptions) {
	const shouldRetry = options.shouldRetry
	return retry(
		args => {
			return ajax({
				...DEFAULT_OPTIONS,
				...options,
				headers: {
					...DEFAULT_HEADERS,
					...options.headers,
				},
				timeout: args.timeout && args.timeout,
			})
		},
		{
			...options,
			shouldRetry: err => {
				// If the user provided a retry function, give it precedence:
				if (shouldRetry) {
					const retry = shouldRetry(err)
					if (retry !== undefined) {
						return retry
					}
				}
				// Check the status code of the request to see if we can retry:
				const status = err && err.status
				if (status) {
					if (status > 0 && status < 400) {
						// Never retry any of those:
						// - 1xx: informational codes
						// - 2xx: success codes
						// - 3xx: redirection codes
						return false
					} else if (status >= 400 && status < 500) {
						// "4xx" are client error codes, don't retry unless:
						// - 408: The server timed out waiting for the rest of
						//        the request from the browser.
						// - 429: Too many requests.
						// - 499: Client closed request.
						return [408, 429, 499].indexOf(status) >= 0
					}
				}
				return true
			},
		}
	)
}

/**
 * Growth modes for the retry delay.
 */
enum RetryGrowth {
	/** No growth, constant delay. */
	None = 1,
	/** Linear growth after each retry. */
	Linear = 2,
	/** Exponential growth after each retry. */
	Exponential = 3,
}

/**
 * Options for retrying an observable operation.
 */
interface RetryOptions {
	/** Maximum number of retries. */
	retries?: number
	/**
	 * Default mode for `delayGrowth` and `timeoutGrowth`.
	 *
	 * If not given, defaults to linear.
	 */
	growth?: RetryGrowth
	/** Initial delay for the first retry. */
	delay?: number
	/** Maximum retry delay or zero for no maximum. */
	delayMax?: number
	/** Growth strategy for `delay` after each attempt. */
	delayGrowth?: RetryGrowth
	/** Initial timeout for the first operation. */
	timeout?: number
	/** Maximum timeout, or zero for no maximum. */
	timeoutMax?: number
	/** Growth strategy for `timeout` after each attempt. */
	timeoutGrowth?: RetryGrowth
	/** Determines if an error should be retried. */
	shouldRetry?: (err: any) => boolean | undefined
}

interface RetryAttempt {
	/** Attempt counter, starting from 1. */
	counter: number
	/** Timeout for this attempt. */
	timeout: number | undefined
}

/**
 * Subscribes to an observable, retrying on errors.
 *
 * Provides both a delay for retrying and a timeout option with configurable
 * growth strategies.
 */
function retry<T>(fnTry: (x: RetryAttempt) => Observable<T>, options: RetryOptions): Observable<T> {
	const retries = options.retries != null ? Math.max(0, options.retries) : DEFAULT_RETRIES

	return new Observable(subscriber => {
		let currentSubscription: Subscription | undefined
		let currentTimeout: number | undefined

		const tryNext = (counter: number) => {
			const isLast = counter - 1 >= retries
			const timeout = getTimeout(counter, options)
			const delay = getDelay(counter, options)
			const start = () => {
				currentTimeout = undefined
				const observable = fnTry({
					counter,
					timeout,
				})
				currentSubscription = observable.subscribe({
					next(value) {
						subscriber.next(value)
					},
					error(err) {
						if (isLast || (options.shouldRetry && options.shouldRetry(err) === false)) {
							subscriber.error(err)
						} else {
							tryNext(counter + 1)
						}
					},
					complete() {
						subscriber.complete()
					},
				})
			}
			if (delay) {
				currentTimeout = setTimeout(() => start(), delay)
			} else {
				currentTimeout = undefined
				start()
			}
		}

		tryNext(1)

		return () => {
			clearTimeout(currentTimeout)
			currentSubscription && currentSubscription.unsubscribe()
		}
	})
}

function getTimeout(counter: number, options: RetryOptions) {
	const timeout = options.timeout != null ? options.timeout : DEFAULT_TIMEOUT
	if (timeout <= 0) {
		return undefined
	}

	const max = options.timeoutMax != null ? options.timeoutMax : DEFAULT_TIMEOUT_MAX
	const clamp = (v?: number) => (v == null || !max || max < 0 ? v : Math.min(v, max))
	const strategy = options.timeoutGrowth || options.growth || RetryGrowth.Linear
	switch (strategy) {
		case RetryGrowth.None:
			return clamp(timeout)
		case RetryGrowth.Linear:
			return clamp(timeout * counter)
		case RetryGrowth.Exponential:
			return clamp(timeout * Math.pow(2, counter - 1))
	}
}

function getDelay(counter: number, options: RetryOptions) {
	const delay = options.delay != null ? options.delay : DEFAULT_DELAY
	if (delay <= 0 || counter <= 1) {
		return 0
	}

	const max = options.delayMax != null ? options.delayMax : DEFAULT_DELAY_MAX
	const clamp = (v: number) => (!max || max < 0 ? v : Math.min(v, max))
	const strategy = options.delayGrowth || options.growth || RetryGrowth.Linear
	switch (strategy) {
		case RetryGrowth.None:
			return clamp(delay)
		case RetryGrowth.Linear:
			return clamp(delay * (counter - 1))
		case RetryGrowth.Exponential:
			return clamp(delay * Math.pow(2, counter - 2))
	}
}
