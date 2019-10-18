import { ActionsObservable, StateObservable } from 'redux-observable'
import { debounceTime, map, mergeMap, takeUntil, filter, catchError, retry } from 'rxjs/operators'
import { push } from 'connected-react-router'
import { merge, of } from 'rxjs'
import { ajax } from 'rxjs/ajax'
import { History } from 'history'

const SEARCH_DEBOUNCE = 200

export interface State {
	/** Current query. */
	query: string

	failed: boolean
	loading: boolean
	data?: DictResult
}

const INITIAL_STATE: State = {
	query: '',
	failed: false,
	loading: false,
}

enum Actions {
	SEARCH = '@dictionary/search',
	REQUEST = '@dictionary/request',
	SUCCESS = '@dictionary/success',
	FAILURE = '@dictionary/failure',
}

/** Dispatch a search query. */
interface Search {
	type: Actions.SEARCH
	args: DictQuery
}

interface Request {
	type: Actions.REQUEST
	args: DictQuery
}

interface Success {
	type: Actions.SUCCESS
	data: {
		query: string
		result?: DictResult
	}
}

interface Failure {
	type: Actions.FAILURE
	query: string
}

export const search = (args: DictQuery): Search => ({
	type: Actions.SEARCH,
	args: args,
})

export const request = (args: DictQuery): Request => ({
	type: Actions.REQUEST,
	args,
})

export const success = (query: string, result?: DictResult): Success => ({
	type: Actions.SUCCESS,
	data: { query, result },
})

export const failure = (query: string): Failure => ({
	type: Actions.FAILURE,
	query,
})

export type Action = Search | Request | Success | Failure

export const dictionaryEpic = (history: History<any>) => (
	action: ActionsObservable<Action>,
	state: StateObservable<State>
) => {
	const searchURL = (q: string) => `/search/${q}`

	const searchEpic = action.ofType<Search>(Actions.SEARCH).pipe(
		debounceTime(SEARCH_DEBOUNCE),
		filter(q => searchURL(q.args.query) !== history.location.pathname),
		map(q => push(searchURL(q.args.query)))
	)

	const requestEpic = action.ofType<Request>(Actions.REQUEST).pipe(
		filter(q => !!q.args.query),
		mergeMap(q =>
			ajax({
				url: '/api/dict/search',
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: {
					query: q.args.query,
					options: {
						limit: 100,
						mode: 'Contains',
					},
				},
			}).pipe(
				retry(2),
				takeUntil(action.ofType(Actions.SEARCH)),
				map(data => success(q.args.query, data.response)),
				catchError(err => {
					console.error(err)
					return of(failure(q.args.query))
				})
			)
		)
	)

	const emptyEpic = action.ofType<Request>(Actions.REQUEST).pipe(
		filter(q => !q.args.query),
		map(q => success(q.args.query))
	)

	return merge(searchEpic, requestEpic, emptyEpic)
}

export default function reducer(state: State = INITIAL_STATE, action: Action): State {
	switch (action.type) {
		case Actions.REQUEST:
			return { ...state, loading: true, query: action.args.query }
		case Actions.SUCCESS:
			return { ...state, loading: false, failed: false, data: action.data.result, query: action.data.query }
		case Actions.FAILURE:
			return { ...state, loading: false, failed: true }
	}
	return state
}

/**
 * Arguments for a dictionary query.
 */
export interface DictQuery {
	query: string
}

/**
 * Root result for a dictionary query.
 */
export interface DictResult {
	id: number
	expression: string
	reading: string
	total: number
	elapsed: number
	terms: DictTerm[]
	tags: { [key: string]: DictTag }
	sources: { [key: string]: DictSource }
}

/**
 * Single term from a dictionary query.
 */
export interface DictTerm {
	id: string
	expression: string
	reading: string
	definition: DictDefinition[]
	romaji: string
	sources: string[]
	forms: DictForm[]
	tags: string[]
	frequency: number | null
	score: number
}

/**
 * English definition for a `DictTerm`.
 */
export interface DictDefinition {
	text: string[]
	info: string[]
	tags: string[]
	link: DictLink[]
}

/**
 * Reference URL for a `DictDefinition`.
 */
export interface DictLink {
	uri: string
	title: string
}

/**
 * Additional forms for a `DictItem`.
 */
export interface DictForm {
	reading: string
	expression: string
	romaji: string
	frequency: number | null
}

/**
 * Tag definition for a dictionary.
 */
export interface DictTag {
	name: string
	category: string
	description: string
	order: number
	source: string
}

/**
 * Source information for a dictionary.
 */
export interface DictSource {
	name: string
	revision: string
}
