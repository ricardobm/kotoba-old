import React from 'react'

import './css/App.scss'
import 'typeface-roboto'
import CssBaseline from '@material-ui/core/CssBaseline'

import * as app from './store/app'
import PingPong from './pages/PingPong'
import { Switch, Route } from 'react-router'
import Todos from './pages/Todos'
import Dictionary from './pages/Dictionary'

import { Container } from '@material-ui/core'
import { ThemeProvider } from '@material-ui/styles'
import MainMenu from './util/MainMenu'
import { AppState } from './store'
import { Action, Dispatch } from 'redux'
import { connect } from 'react-redux'
import Home from './pages/Home'

import { createAppTheme } from './base/theme'
import * as nav from './base/nav'
import { Location } from 'history'

interface IDispatch {}

interface IState extends app.State {
	location: Location<any>
}

interface IProps extends IState, IDispatch {}

const App: React.FC<IProps> = self => {
	// This is the main theme for the application:
	return (
		<ThemeProvider theme={createAppTheme()}>
			<MainMenu title={self.title} location={self.location}>
				<Container maxWidth="lg">
					<CssBaseline />
					<Switch>
						<Route exact path={nav.homeURL}>
							<Home />
						</Route>
						<Route path={nav.pingPongURL}>
							<PingPong />
						</Route>
						<Route path={nav.todoURL}>
							<Todos />
						</Route>
						<Route path={`${nav.dictionaryURL}/:query?`}>
							<Dictionary />
						</Route>
					</Switch>
				</Container>
			</MainMenu>
		</ThemeProvider>
	)
}

const mapStateToProps = (state: AppState): IState => {
	return {
		...state.app,
		location: state.router.location,
	}
}

const mapDispatchToProps = (dispatch: Dispatch<Action>): IDispatch => ({})

export default connect(
	mapStateToProps,
	mapDispatchToProps,
	null,
	{ pure: false }
)(App)
