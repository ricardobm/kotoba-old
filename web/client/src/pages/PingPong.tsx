import React from 'react'

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

const PingPong: React.FC<IProps> = self => {
	const onServe = () => self.serve()
	const onReset = () => self.reset()
	return (
		<div className="App">
			<p>{self.running ? self.label : 'Waiting serve...'}</p>
			<button onClick={onServe} disabled={self.running}>
				Serve
			</button>
			&nbsp;
			<button onClick={onReset} disabled={!self.running}>
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
