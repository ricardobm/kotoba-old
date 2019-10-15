import { Dispatch } from 'redux'
import { ofType, ActionsObservable, StateObservable } from 'redux-observable'
import { debounceTime, map, mergeMap, takeUntil, filter, catchError, retry } from 'rxjs/operators'
import { push } from 'connected-react-router'
import { merge, of } from 'rxjs'
import { ajax } from 'rxjs/ajax'

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
	data?: DictResult
}

interface Failure {
	type: Actions.FAILURE
}

export const search = (args: DictQuery): Search => ({
	type: Actions.SEARCH,
	args: args,
})

export const request = (args: DictQuery): Request => ({
	type: Actions.REQUEST,
	args,
})

export const success = (data?: DictResult): Success => ({
	type: Actions.SUCCESS,
	data,
})

export const failure = (): Failure => ({
	type: Actions.FAILURE,
})

export type Action = Search | Request | Success | Failure

export const dictionaryEpic = (action: ActionsObservable<Action>, state: StateObservable<State>) => {
	const searchEpic = action.ofType<Search>(Actions.SEARCH).pipe(
		debounceTime(250),
		map(q => request(q.args))
	)

	const locationEpic = action.ofType<Request>(Actions.REQUEST).pipe(map(q => push(`/search/${q.args.query}`)))

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
				map(data => success(data.response)),
				catchError(err => {
					console.error(err)
					return of(failure())
				})
			)
		)
	)

	const emptyEpic = action.ofType<Request>(Actions.REQUEST).pipe(
		filter(q => !q.args.query),
		map(() => success())
	)

	return merge(searchEpic, locationEpic, requestEpic, emptyEpic)
}

export default function reducer(state: State = INITIAL_STATE, action: Action): State {
	switch (action.type) {
		case Actions.SEARCH:
			return { ...state, query: action.args.query }
		case Actions.REQUEST:
			return { ...state, loading: true }
		case Actions.SUCCESS:
			return { ...state, loading: false, failed: false, data: action.data }
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
	source: number
}

/**
 * Source information for a dictionary.
 */
export interface DictSource {
	name: string
	revision: string
}
