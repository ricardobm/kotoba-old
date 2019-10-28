import 'react-app-polyfill/ie11'
import 'react-app-polyfill/stable'

import React from 'react'

import whyDidYouRender from '@welldone-software/why-did-you-render'

import ReactDOM from 'react-dom'
import { Provider } from 'react-redux'
import App from './App'
import * as serviceWorker from './serviceWorker'

import { Router } from 'react-router-dom'
import { ConnectedRouter } from 'connected-react-router'
import { configureStore, history } from './store'

import './css/index.scss'

whyDidYouRender(React)

ReactDOM.render(
	<Provider store={configureStore()}>
		<ConnectedRouter history={history}>
			<Router history={history}>
				<App />
			</Router>
		</ConnectedRouter>
	</Provider>,
	document.getElementById('root')
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
