import 'react-app-polyfill/ie11'
import 'react-app-polyfill/stable'

import React from 'react'
import ReactDOM from 'react-dom'
import { Provider } from 'react-redux'
import App from './App'
import * as serviceWorker from './serviceWorker'
import { BrowserRouter } from 'react-router-dom'

import { configureStore, history } from './store'

import './css/index.scss'
import { ConnectedRouter } from 'connected-react-router'

ReactDOM.render(
	<Provider store={configureStore()}>
		<ConnectedRouter history={history}>
			<BrowserRouter>
				<App />
			</BrowserRouter>
		</ConnectedRouter>
	</Provider>,
	document.getElementById('root')
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
