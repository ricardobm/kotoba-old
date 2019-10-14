import React, { useEffect } from 'react'

import * as ping_pong from '../store/ping_pong'
import { AppState } from '../store'
import { connect } from 'react-redux'
import { Dispatch, Action } from 'redux'
import { Link } from 'react-router-dom'

interface IDispatch {
	serve: () => ping_pong.Action
	reset: () => ping_pong.Action
}

interface IState {
	label: string
	running: boolean
}

interface IProps extends IDispatch, IState {}

const PingPong: React.FC<IProps> = state => {
	const onServe = () => state.serve()
	const onReset = () => state.reset()

	useEffect(() => {
		return () => {
			state.reset()
		}
	}, [])

	return (
		<div className="App">
			<p>{state.running ? state.label : 'Waiting serve...'}</p>
			<button onClick={onServe} disabled={state.running}>
				Serve
			</button>
			&nbsp;
			<button onClick={onReset} disabled={!state.running}>
				Reset
			</button>
			<Link to="/" className="App-link">
				Go back
			</Link>
		</div>
	)
}

const mapStateToProps = (state: AppState): IState => ({
	label: state.ping_pong.ping ? 'PING' : 'PONG',
	running: state.ping_pong.ping != null,
})

const mapDispatchToProps = (dispatch: Dispatch<Action>): IDispatch => ({
	serve: () => dispatch(ping_pong.serve()),
	reset: () => dispatch(ping_pong.reset()),
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(PingPong)
