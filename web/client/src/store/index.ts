import { applyMiddleware, combineReducers, createStore, compose } from 'redux'
import thunk from 'redux-thunk'
import { createEpicMiddleware, combineEpics } from 'redux-observable'
import { connectRouter, routerMiddleware } from 'connected-react-router'
import { createBrowserHistory } from 'history'

import todo from './todo'
import ping_pong, { pingPongEpic } from './ping_pong'

export const history = createBrowserHistory()

const rootReducer = combineReducers({
	router: connectRouter(history),
	todo,
	ping_pong,
})

const rootEpic = combineEpics(pingPongEpic)

export type AppState = ReturnType<typeof rootReducer>

const epicMiddleware = createEpicMiddleware()
const composeEnhancers: <R>(a: R) => R = (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ || compose

export function configureStore() {
	const router = routerMiddleware(history)
	const store = createStore(rootReducer, composeEnhancers(applyMiddleware(router, thunk, epicMiddleware)))
	epicMiddleware.run(rootEpic as any)
	return store
}
