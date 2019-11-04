import { connect } from 'react-redux'
import { AppState } from '../store'
import { Action, Dispatch } from 'redux'
import { ThunkDispatch } from 'redux-thunk'
import React, { useEffect, useLayoutEffect, useRef } from 'react'
import { makeStyles, createStyles, Theme } from '@material-ui/core'
import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@material-ui/lab'

import AutoSizer from 'react-virtualized-auto-sizer'

import * as app from '../store/app'
import Editor from '../ui/Editor'

import SaveIcon from '@material-ui/icons/Save'
import FileCopyIcon from '@material-ui/icons/FileCopy'
import CancelIcon from '@material-ui/icons/Cancel'
import DeleteIcon from '@material-ui/icons/Delete'
import clsx from 'clsx'
import { doc } from '../ui/edit'

interface IState {}

interface IDispatch {
	dispatch: Dispatch<Action>
}

interface IProps extends IState, IDispatch {}

const useStyles = makeStyles((theme: Theme) => {
	return createStyles({
		root: {
			height: '100%',
		},
		dial: {
			position: 'absolute',
			bottom: theme.spacing(2),
			right: theme.spacing(4),
			opacity: 0.3,
		},
		dialOpen: {
			opacity: 1.0,
		},
		dialAction: {
			minWidth: 200,
		},
	})
})

const Wiki: React.FC<IProps> = self => {
	useEffect(() => {
		self.dispatch(app.setTitle('Editor'))
	}, [])

	const [open, setOpen] = React.useState(false)
	const handleOpen = () => setOpen(true)
	const handleClose = () => setOpen(false)

	const action = (name: string, icon: React.ReactNode) => (
		<SpeedDialAction icon={icon} tooltipTitle={name.replace(/ /g, '\xA0')} tooltipOpen />
	)

	const document: doc.DocNode[] = [
		doc.paragraph('p1', [
			doc.text('txt1', 'Hello there, '),
			doc.em('em1', [doc.strong('s1', [doc.text('txt2', 'YOUR NAME HERE')])]),
			doc.text('txt3', '!'),
		]),
	]

	const classes = useStyles()
	return (
		<div className={classes.root}>
			<AutoSizer disableWidth>
				{({ height }) => {
					return <Editor height={height} document={document} />
				}}
			</AutoSizer>
			<SpeedDial
				ariaLabel="Actions"
				className={clsx(classes.dial, {
					[classes.dialOpen]: open,
				})}
				icon={<SpeedDialIcon />}
				open={open}
				onOpen={handleOpen}
				onClose={handleClose}
			>
				{action('Save', <SaveIcon />)}
				{action('Copy', <FileCopyIcon />)}
				{action('Cancel', <CancelIcon />)}
				{action('Delete', <DeleteIcon />)}
			</SpeedDial>
		</div>
	)
}

const mapStateToProps = (state: AppState): IState => ({})

const mapDispatchToProps = (dispatch: ThunkDispatch<any, any, Action>): IDispatch => ({
	dispatch,
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(Wiki)
