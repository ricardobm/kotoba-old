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
	Hidden,
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

export interface IProps {
	title: string
	children?: React.ReactNode
}

const drawerWidth = 250

const appMenuToggleClass = 'app-menu-toggle-button'

const useStyles = makeStyles((theme: Theme) => {
	const menuButtonWidth = theme.spacing(7) + 1
	const isMobile = theme.breakpoints.down('sm')
	return createStyles({
		root: {
			display: 'flex',
			minHeight: '100%',
		},

		// Top application bar
		appBar: {
			zIndex: theme.zIndex.drawer + 1,
			transition: theme.transitions.create(['width', 'margin', 'padding'], {
				easing: theme.transitions.easing.sharp,
				duration: theme.transitions.duration.leavingScreen,
			}),
			[`& button.${appMenuToggleClass}`]: {
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
			[`& button.${appMenuToggleClass}`]: {
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
			width: menuButtonWidth,
			[isMobile]: {
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
			flexGrow: 0,
			flexShrink: 0,
		},
		content: {
			display: 'flex',
			minHeight: '100%',
			flexDirection: 'column',
			paddingLeft: menuButtonWidth,
			flexGrow: 1,
			[isMobile]: {
				paddingLeft: theme.spacing(3),
			},
		},
	})
})

const MainMenu: React.FC<IProps> = props => {
	const [open, setOpen] = React.useState(false)
	const openDrawer = () => setOpen(true)
	const closeDrawer = () => setOpen(false)

	const dispatch = useDispatch()

	const go = (fn: (x: any) => void, isMobile: boolean) => {
		fn(dispatch)
		if (isMobile) {
			closeDrawer()
		}
	}

	const goHome = (isMobile: boolean) => go(nav.goHome, isMobile)
	const goPages = (isMobile: boolean) => go(nav.goWiki, isMobile)
	const goDecks = (isMobile: boolean) => go(nav.goPingPong, isMobile)
	const goDictionary = (isMobile: boolean) => go(nav.goDictionary, isMobile)

	const theme = useTheme()
	const classes = useStyles()

	const menuItems = (isMobile: boolean) => (
		<React.Fragment>
			<div className={classes.drawerHeader}>
				<IconButton onClick={closeDrawer}>
					{theme.direction === 'ltr' ? <ChevronLeftIcon /> : <ChevronRightIcon />}
				</IconButton>
			</div>
			<Divider />
			<List>
				<ListItem button onClick={() => goHome(isMobile)}>
					<ListItemIcon>
						<HomeIcon />
					</ListItemIcon>
					<ListItemText primary="Home" />
				</ListItem>
				<ListItem button onClick={() => goPages(isMobile)}>
					<ListItemIcon>
						<MenuBookIcon />
					</ListItemIcon>
					<ListItemText primary="Pages" />
				</ListItem>
				<ListItem button onClick={() => goDecks(isMobile)}>
					<ListItemIcon>
						<LoyaltyIcon />
					</ListItemIcon>
					<ListItemText primary="Decks" />
				</ListItem>
				<ListItem button onClick={() => goDictionary(isMobile)}>
					<ListItemIcon>
						<TranslateIcon />
					</ListItemIcon>
					<ListItemText primary="Dictionary" />
				</ListItem>
			</List>
		</React.Fragment>
	)

	const isDesktop = { smUp: true }
	const isMobile = { xsDown: true }

	const drawerClasses = {
		className: clsx(classes.drawer, {
			[classes.drawerOpen]: open,
			[classes.drawerClose]: !open,
		}),
		classes: {
			paper: clsx({
				[classes.drawerOpen]: open,
				[classes.drawerClose]: !open,
			}),
		},
	}

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
						className={clsx([classes.menuButton, appMenuToggleClass], {
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

			<Hidden {...isMobile} implementation="js">
				<Drawer anchor="left" variant="permanent" open {...drawerClasses}>
					{menuItems(false)}
				</Drawer>
			</Hidden>

			<Hidden {...isDesktop} implementation="js">
				<Drawer
					anchor="left"
					variant="temporary"
					open={open}
					{...drawerClasses}
					onEscapeKeyDown={closeDrawer}
					onBackdropClick={closeDrawer}
					ModalProps={{
						keepMounted: true, // Better open performance on mobile.
					}}
				>
					{menuItems(true)}
				</Drawer>
			</Hidden>

			<main className={classes.content}>
				<div className={classes.toolbar} />
				{props.children}
			</main>
		</div>
	)
}

export default MainMenu
