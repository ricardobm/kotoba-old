import React from 'react'
import logo from './logo.svg'

import './css/App.scss'
import 'typeface-roboto'
import CssBaseline from '@material-ui/core/CssBaseline'

import PingPong from './pages/PingPong'
import { Switch, Route } from 'react-router'
import Link from '@material-ui/core/Link'
import Todos from './pages/Todos'
import Dictionary from './pages/Dictionary'

import { TiMediaEject } from 'react-icons/ti'

import { AccessAlarm, Eject } from '@material-ui/icons'

import { makeStyles, Typography, Container, createMuiTheme, Button } from '@material-ui/core'
import { ThemeProvider } from '@material-ui/styles'
import { lightBlue, blueGrey, red, deepOrange } from '@material-ui/core/colors'
import LinkTo from './util/LinkTo'

const App: React.FC = () => {
	// This is the main theme for the application:
	const theme = createMuiTheme({
		typography: {
			fontSize: 16,
		},
		spacing: 4,
		palette: {
			type: 'dark',
			text: {
				primary: blueGrey['50'],
				secondary: blueGrey['200'],
			},
			error: {
				main: red['600'],
			},
			primary: {
				main: lightBlue['400'],
			},
			secondary: {
				main: deepOrange['400'],
			},
		},
	})

	return (
		<ThemeProvider theme={theme}>
			<Container maxWidth="lg">
				<CssBaseline />
				<Switch>
					<Route exact path="/">
						<Home />
					</Route>
					<Route path="/ping_pong">
						<PingPong />
					</Route>
					<Route path="/todo" onEnter>
						<Todos />
					</Route>
					<Route path="/search/:query?">
						<Dictionary />
					</Route>
				</Switch>
			</Container>
		</ThemeProvider>
	)
}

const useStyles = makeStyles({
	root: {
		width: '100%',
		maxWidth: 500,
	},
})

const Home: React.FC = () => {
	const classes = useStyles()
	return (
		<div className="App">
			<header className="App-header">
				<div className={classes.root}>
					<Typography variant="h1" gutterBottom>
						My heading!
						<span className="japanese">君の知らない物語</span>
					</Typography>
					<Typography color="textPrimary">
						Hello there from{' '}
						<Typography component="span" color="textSecondary">
							( SECONDARY )
						</Typography>
						<Typography component="span" color="error">
							( ERROR )
						</Typography>
						<Typography component="span" color="primary">
							( PRIMARY )
						</Typography>
						<Typography component="span" color="secondary">
							( SECONDARY )
						</Typography>
						<Button href="google.com">Google</Button>
						!.
					</Typography>
					<p className="japanese">君の知らない物語</p>
					<p className="japanese" style={{ fontSize: '0.5em' }}>
						君の知らない物語
					</p>
					<p className="japanese" style={{ fontSize: '0.4em' }}>
						君の知らない物語
					</p>
					<p className="japanese" style={{ fontSize: '0.3em' }}>
						君の知らない物語
					</p>
					<p className="japanese">
						「約物半角専用のWebフォント」を優先的に当てることによって、
						Webテキストの日本語に含まれる約物を半角にすることができました。
						例えば「かっこ」や『二重かっこ』、
						【バッジに使いそうなかっこ】などを半角にできます。ウェイトは7種類。Noto Sans
						Japaneseに沿っています。
					</p>
					<p className="japanese" style={{ fontSize: '0.3em' }}>
						「約物半角専用のWebフォント」を優先的に当てることによって、
						Webテキストの日本語に含まれる約物を半角にすることができました。
						例えば「かっこ」や『二重かっこ』、
						【バッジに使いそうなかっこ】などを半角にできます。ウェイトは7種類。Noto Sans
						Japaneseに沿っています。
					</p>
				</div>
				<TiMediaEject />
				<AccessAlarm /> <Eject />
				<img src={logo} className="App-logo" alt="logo" />
				<p>
					Edit <code>src/App.tsx</code> and save to reload.
				</p>
				<Link href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
					Learn React
				</Link>
				<LinkTo to="/todo">TODO</LinkTo>
				<LinkTo to="/search">Search</LinkTo>
				<LinkTo to="/ping_pong">Ping pong</LinkTo>
			</header>
		</div>
	)
}

export default App
