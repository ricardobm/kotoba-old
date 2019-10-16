import React from 'react'

import {
	AppBar,
	Toolbar,
	Typography,
	IconButton,
	Drawer,
	Divider,
	Theme,
	createStyles,
	makeStyles,
	useTheme,
	List,
	ListItem,
	ListItemIcon,
	ListItemText,
} from '@material-ui/core'

import clsx from 'clsx'

import MenuIcon from '@material-ui/icons/Menu'
import HomeIcon from '@material-ui/icons/Home'
import TranslateIcon from '@material-ui/icons/Translate'
import MenuBookIcon from '@material-ui/icons/MenuBook'
import LoyaltyIcon from '@material-ui/icons/Loyalty'
import ChevronLeftIcon from '@material-ui/icons/ChevronLeft'
import ChevronRightIcon from '@material-ui/icons/ChevronRight'

import * as nav from '../base/nav'
import { useDispatch } from 'react-redux'
import { Location } from 'history'

export interface IProps {
	title: string
	location: Location<any>
	children?: React.ReactNode
}

const drawerWidth = 250

const useStyles = makeStyles((theme: Theme) =>
	createStyles({
		root: {
			display: 'flex',
		},

		// Top application bar
		appBar: {
			zIndex: theme.zIndex.drawer + 1,
			transition: theme.transitions.create(['width', 'margin', 'padding'], {
				easing: theme.transitions.easing.sharp,
				duration: theme.transitions.duration.leavingScreen,
			}),
			'& button:first-of-type': {
				transition: theme.transitions.create(['margin', 'opacity'], {
					easing: theme.transitions.easing.sharp,
					duration: theme.transitions.duration.leavingScreen,
				}),
			},
		},
		appBarShift: {
			marginLeft: drawerWidth,
			width: `calc(100% - ${drawerWidth}px)`,
			transition: theme.transitions.create(['width', 'margin'], {
				easing: theme.transitions.easing.sharp,
				duration: theme.transitions.duration.enteringScreen,
			}),
			'& button:first-of-type': {
				marginRight: -36,
				transition: theme.transitions.create(['margin', 'opacity'], {
					easing: theme.transitions.easing.sharp,
					duration: theme.transitions.duration.enteringScreen,
				}),
			},
		},

		// Styles for the menu button
		menuButton: {
			marginRight: 36,
		},
		hide: {
			opacity: 0,
		},

		// Left drawer
		drawer: {
			flexShrink: 0,
			whiteSpace: 'nowrap',
			width: drawerWidth,
		},
		drawerOpen: {
			width: drawerWidth,
			transition: theme.transitions.create('width', {
				easing: theme.transitions.easing.sharp,
				duration: theme.transitions.duration.enteringScreen,
			}),
		},
		drawerClose: {
			transition: theme.transitions.create('width', {
				easing: theme.transitions.easing.sharp,
				duration: theme.transitions.duration.leavingScreen,
			}),
			overflowX: 'hidden',
			width: theme.spacing(7) + 1,
			[theme.breakpoints.down('sm')]: {
				width: 0,
				marginLeft: -1, // hide the right border when collapsed
			},
		},
		drawerPaper: {
			width: drawerWidth,
		},
		drawerHeader: {
			display: 'flex',
			alignItems: 'center',
			justifyContent: 'flex-end',
			paddingRight: theme.spacing(1),
			...theme.mixins.toolbar,
			marginBottom: theme.spacing(3),
		},

		// Content
		toolbar: {
			...theme.mixins.toolbar,
		},
		content: {
			padding: theme.spacing(3),
			flexGrow: 1,
		},
	})
)

const MainMenu: React.FC<IProps> = props => {
	const [open, setOpen] = React.useState(false)
	const openDrawer = () => setOpen(true)
	const closeDrawer = () => setOpen(false)

	const dispatch = useDispatch()

	const goHome = () => nav.goHome(dispatch)
	const goPages = () => nav.goTodo(dispatch)
	const goDecks = () => nav.goPingPong(dispatch)
	const goDictionary = () => nav.goDictionary(dispatch)

	const theme = useTheme()
	const classes = useStyles()
	return (
		<div className={classes.root}>
			<AppBar
				position="fixed"
				className={clsx(classes.appBar, {
					[classes.appBarShift]: open,
				})}
			>
				<Toolbar>
					<IconButton
						color="inherit"
						onClick={openDrawer}
						className={clsx(classes.menuButton, {
							[classes.hide]: open,
						})}
					>
						<MenuIcon />
					</IconButton>
					<Typography variant="h6" noWrap>
						{props.title}
					</Typography>
				</Toolbar>
			</AppBar>
			<Drawer
				anchor="left"
				variant="permanent"
				open={open}
				className={clsx(classes.drawer, {
					[classes.drawerOpen]: open,
					[classes.drawerClose]: !open,
				})}
				classes={{
					paper: clsx({
						[classes.drawerOpen]: open,
						[classes.drawerClose]: !open,
					}),
				}}
			>
				<div className={classes.drawerHeader}>
					<IconButton onClick={closeDrawer}>
						{theme.direction === 'ltr' ? <ChevronLeftIcon /> : <ChevronRightIcon />}
					</IconButton>
				</div>
				<Divider />
				<List>
					<ListItem button onClick={goHome}>
						<ListItemIcon>
							<HomeIcon />
						</ListItemIcon>
						<ListItemText primary="Home" />
					</ListItem>
					<ListItem button onClick={goPages}>
						<ListItemIcon>
							<MenuBookIcon />
						</ListItemIcon>
						<ListItemText primary="Pages" />
					</ListItem>
					<ListItem button onClick={goDecks}>
						<ListItemIcon>
							<LoyaltyIcon />
						</ListItemIcon>
						<ListItemText primary="Decks" />
					</ListItem>
					<ListItem button onClick={goDictionary}>
						<ListItemIcon>
							<TranslateIcon />
						</ListItemIcon>
						<ListItemText primary="Dictionary" />
					</ListItem>
				</List>
			</Drawer>
			<main className={classes.content}>
				<div className={classes.toolbar} />
				{props.children}
			</main>
		</div>
	)
}

export default MainMenu
