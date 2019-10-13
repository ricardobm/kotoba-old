import { Epic, ofType } from 'redux-observable'
import { mapTo, delay, takeUntil, switchMap, startWith } from 'rxjs/operators'
import { merge } from 'rxjs'

export interface State {
	ping: boolean | null
}

const INITIAL_STATE: State = {
	ping: null,
}

enum Actions {
	PING = '@ping_pong/ping',
	PONG = '@ping_pong/pong',
	SERVE = '@ping_pong/serve',
	RESET = '@ping_pong/reset',
}

interface Ping {
	type: Actions.PING
}

interface Pong {
	type: Actions.PONG
}

interface Serve {
	type: Actions.SERVE
}

interface Reset {
	type: Actions.RESET
}

export type Action = Ping | Pong | Serve | Reset

export const ping = () => ({ type: Actions.PING } as Ping)

export const pong = () => ({ type: Actions.PONG } as Pong)

export const serve = () => ({ type: Actions.SERVE } as Serve)

export const reset = () => ({ type: Actions.RESET } as Reset)

export const pingPongEpic: Epic<Action, Action, State> = (action, state) => {
	const serve = () => {
		const player1 = action.pipe(
			ofType(Actions.PING),
			delay(1000),
			takeUntil(action.pipe(ofType(Actions.RESET))),
			mapTo(pong())
		)
		const player2 = action.pipe(
			ofType(Actions.PONG),
			delay(1000),
			takeUntil(action.pipe(ofType(Actions.RESET))),
			mapTo(ping())
		)
		return merge(player1, player2).pipe(startWith(ping()))
	}

	return action.pipe(
		ofType(Actions.SERVE),
		switchMap(() => serve())
	)
}

export default function reducer(state: State = INITIAL_STATE, action: Action) {
	switch (action.type) {
		case Actions.PING:
			return { ...state, ping: true }
		case Actions.PONG:
			return { ...state, ping: false }
		case Actions.RESET:
			return { ...state, ping: null }
		default:
			return state
	}
}
