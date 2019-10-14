import { Dispatch } from 'redux'
import { ofType, ActionsObservable, StateObservable } from 'redux-observable'
import { debounceTime, map, tap } from 'rxjs/operators'
import { push } from 'connected-react-router'

export interface State {
	/** Current query. */
	query: string
}

const INITIAL_STATE: State = {
	query: '',
}

enum Actions {
	SEARCH = '@dictionary/search',
	START_QUERY = '@dictionary/start_query',
}

/** Dispatch a search query. */
interface Search {
	type: Actions.SEARCH
	args: DictQuery
}

/** Actually starts loading a search query. */
interface StartQuery {
	type: Actions.START_QUERY
	args: DictQuery
}

export const search = (args: DictQuery): Search => ({
	type: Actions.SEARCH,
	args: args,
})

export const startQuery = (args: DictQuery) => {
	return async (dispatch: Dispatch) => {
		console.log(args)
		dispatch(push(`/search/${args.query}`))
		return dispatch({ type: Actions.START_QUERY, args })
	}
}

export type Action = Search | StartQuery

export const dictionaryEpic = (action: ActionsObservable<Action>, state: StateObservable<State>) => {
	return action.pipe(
		ofType(Actions.SEARCH),
		debounceTime(250),
		tap(q => console.log(q)),
		map(q => startQuery(q.args))
	)
}

export default function reducer(state: State = INITIAL_STATE, action: Action): State {
	switch (action.type) {
		case Actions.SEARCH:
			return { ...state, query: action.args.query }
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
