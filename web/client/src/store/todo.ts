import { Dispatch } from 'redux'

export interface State {
	readonly title: string
	readonly todos: string[]
	readonly loading: boolean
	readonly loaded: boolean
	readonly error?: string
}

const INITIAL_STATE: State = {
	title: 'TODO',
	todos: [],
	loading: false,
	loaded: false,
}

enum Actions {
	ADD = '@todo/add',
	DEL = '@todo/del',
	LOADING = '@todo/loading',
	REQUEST = '@todo/request',
	SUCCESS = '@todo/success',
	FAILURE = '@todo/fAILURE',
	SET_TITLE = '@todo/setTitle',
}

interface Add {
	type: Actions.ADD
	text: string
}

interface Del {
	type: Actions.DEL
	index: number
}

interface Loading {
	type: Actions.LOADING
}

interface Request {
	type: Actions.REQUEST
}

interface Success {
	type: Actions.SUCCESS
	data: string[]
}

interface Failure {
	type: Actions.FAILURE
	reason: string
}

interface SetTitle {
	type: Actions.SET_TITLE
	title: string
}

export type Action = Add | Del | SetTitle | Loading | Request | Success | Failure

export const add = (text: string) =>
	({
		type: Actions.ADD,
		text,
	} as Add)

export const del = (index: number) =>
	({
		type: Actions.DEL,
		index,
	} as Del)

export const setTitle = (title: string) =>
	({
		type: Actions.SET_TITLE,
		title,
	} as SetTitle)

export const request = () => {
	return async (dispatch: Dispatch) => {
		dispatch({ type: Actions.LOADING } as Loading)
		try {
			const data = (await fetch('/api/list').then(data => data.json())) as any[]
			return dispatch({ type: Actions.SUCCESS, data: data.map(x => x.text) } as Success)
		} catch (e) {
			return dispatch({ type: Actions.FAILURE, reason: e.message } as Failure)
		}
	}
}

export default function reducer(state = INITIAL_STATE, action: Action) {
	switch (action.type) {
		case Actions.ADD: {
			const todos = state.todos.concat([action.text])
			return { ...state, todos }
		}
		case Actions.DEL: {
			const todos = state.todos.slice()
			todos.splice(action.index, 1)
			return { ...state, todos }
		}
		case Actions.SET_TITLE: {
			return { ...state, title: action.title }
		}
		case Actions.LOADING: {
			return { ...state, loading: true }
		}
		case Actions.SUCCESS: {
			return { ...state, todos: action.data, error: undefined, loaded: true, loading: false }
		}
		case Actions.FAILURE: {
			return { ...state, error: `Failed to load: ${action.reason}`, loaded: true, loading: false }
		}
	}
	return state
}
