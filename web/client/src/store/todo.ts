export interface State {
	readonly title: string
	readonly todos: string[]
}

const INITIAL_STATE: State = {
	title: 'TODO',
	todos: ['sample todo'],
}

enum Actions {
	ADD = '@todo/add',
	DEL = '@todo/del',
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

interface SetTitle {
	type: Actions.SET_TITLE
	title: string
}

type Action = Add | Del | SetTitle

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
	}
	return state
}
