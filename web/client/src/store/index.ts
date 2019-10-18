import { applyMiddleware, combineReducers, createStore, compose } from 'redux'
import thunk from 'redux-thunk'
import { createEpicMiddleware, combineEpics } from 'redux-observable'
import { connectRouter, routerMiddleware } from 'connected-react-router'
import { createBrowserHistory } from 'history'
import { createLogger } from 'redux-logger'

import app from './app'
import home from './home'
import todo from './todo'
import ping_pong, { pingPongEpic } from './ping_pong'
import dictionary, { dictionaryEpic } from './dictionary'

const ENABLE_LOGGER = false
const ENABLE_LOGGER_DIFF = false

export const history = createBrowserHistory()

const rootReducer = combineReducers({
	router: connectRouter(history),
	app,
	home,
	todo,
	ping_pong,
	dictionary,
})

const rootEpic = combineEpics(pingPongEpic, dictionaryEpic(history))

export type AppState = ReturnType<typeof rootReducer>

const epicMiddleware = createEpicMiddleware()
const composeEnhancers: <R>(a: R) => R = (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ || compose

export function configureStore() {
	const router = routerMiddleware(history)
	const middlewares = [router, thunk, epicMiddleware]
	if (ENABLE_LOGGER) {
		const logger = createLogger({
			diff: ENABLE_LOGGER_DIFF,
		})
		middlewares.push(logger)
	}

	const store = createStore(rootReducer, composeEnhancers(applyMiddleware(...middlewares)))
	epicMiddleware.run(rootEpic as any)
	return store
}
