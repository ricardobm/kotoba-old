import { applyMiddleware, combineReducers, createStore, compose } from 'redux'
import thunk from 'redux-thunk'
import { createEpicMiddleware, combineEpics } from 'redux-observable'

import todo from './todo'
import ping_pong, { pingPongEpic } from './ping_pong'

const rootReducer = combineReducers({ todo, ping_pong })
const rootEpic = combineEpics(pingPongEpic)

export type AppState = ReturnType<typeof rootReducer>

const epicMiddleware = createEpicMiddleware()
const composeEnhancers: <R>(a: R) => R = (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ || compose

export default function configureStore() {
	const store = createStore(rootReducer, composeEnhancers(applyMiddleware(thunk, epicMiddleware)))
	epicMiddleware.run(rootEpic as any)
	return store
}
