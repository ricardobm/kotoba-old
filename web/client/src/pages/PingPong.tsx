import React, { useEffect } from 'react'

import * as app from '../store/app'
import * as ping_pong from '../store/ping_pong'
import { AppState } from '../store'
import { connect } from 'react-redux'
import { Dispatch, Action } from 'redux'
import LinkTo from '../util/LinkTo'

interface IDispatch {
	dispatch: Dispatch<Action>
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

	useEffect(() => {
		self.dispatch(app.setTitle('Ping Pong'))
		return () => {
			self.reset()
		}
	}, []) // eslint-disable-line react-hooks/exhaustive-deps

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
			<LinkTo to="/">Go back</LinkTo>
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
	dispatch,
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(PingPong)
