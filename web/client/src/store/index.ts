import { applyMiddleware, combineReducers, createStore, compose } from 'redux'
import thunk from 'redux-thunk'
import { createEpicMiddleware, combineEpics } from 'redux-observable'
import { connectRouter, routerMiddleware } from 'connected-react-router'
import { createBrowserHistory } from 'history'

import app from './app'
import home from './home'
import todo from './todo'
import ping_pong, { pingPongEpic } from './ping_pong'
import dictionary, { dictionaryEpic } from './dictionary'

export const history = createBrowserHistory()

const rootReducer = combineReducers({
	router: connectRouter(history),
	app,
	home,
	todo,
	ping_pong,
	dictionary,
})

const rootEpic = combineEpics(pingPongEpic, dictionaryEpic)

export type AppState = ReturnType<typeof rootReducer>

const epicMiddleware = createEpicMiddleware()
const composeEnhancers: <R>(a: R) => R = (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ || compose

export function configureStore() {
	const w = window as any
	w.history_main = history
	const router = routerMiddleware(history)
	const store = createStore(rootReducer, composeEnhancers(applyMiddleware(router, thunk, epicMiddleware)))
	epicMiddleware.run(rootEpic as any)
	return store
}
