import { Dispatch, Action } from 'redux'
import { push } from 'connected-react-router'

export const homeURL = '/'
export const todoURL = '/todo'
export const dictionaryURL = '/search'
export const pingPongURL = '/ping-pong'

export function goHome(dispatch: Dispatch<Action>) {
	dispatch(push(homeURL))
}

export function goTodo(dispatch: Dispatch<Action>) {
	dispatch(push(todoURL))
}

export function goDictionary(dispatch: Dispatch<Action>) {
	dispatch(push(dictionaryURL))
}

export function goPingPong(dispatch: Dispatch<Action>) {
	dispatch(push(pingPongURL))
}
