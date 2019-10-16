export interface State {
	title: string
}

const INITIAL_STATE: State = {
	title: 'Home',
}

enum Actions {
	SET_TITLE = '@app/set_title',
}

interface SetTitle {
	type: Actions.SET_TITLE
	title: string
}

export const setTitle = (title: string): SetTitle => ({ type: Actions.SET_TITLE, title })

export type Action = SetTitle

export default function reducer(state: State = INITIAL_STATE, action: Action): State {
	switch (action.type) {
		case Actions.SET_TITLE:
			document.title = `Hongo - ${action.title}`
			return { ...state, title: action.title }
	}
	return state
}
