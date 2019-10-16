import React from 'react'
import logo from './logo.svg'
import './css/App.scss'

import PingPong from './pages/PingPong'
import { Switch, Route } from 'react-router'
import { Link } from 'react-router-dom'
import Todos from './pages/Todos'
import Dictionary from './pages/Dictionary'

import { TiMediaEject } from 'react-icons/ti'

import { AccessAlarm, Eject } from '@material-ui/icons'

import 'typeface-roboto'
import { makeStyles, Typography } from '@material-ui/core'

class App extends React.Component {
	render() {
		return (
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
		)
	}
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
				<a className="App-link" href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
					Learn React
				</a>
				<Link to="/todo" className="App-link">
					TODO
				</Link>
				<Link to="/search" className="App-link">
					Search
				</Link>
				<Link to="/ping_pong" className="App-link">
					Ping pong
				</Link>
			</header>
		</div>
	)
}

export default App
