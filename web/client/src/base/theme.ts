import { createMuiTheme } from '@material-ui/core'

import blueGrey from '@material-ui/core/colors/blueGrey'
import deepOrange from '@material-ui/core/colors/deepOrange'
import lightBlue from '@material-ui/core/colors/lightBlue'
import amber from '@material-ui/core/colors/amber'
import red from '@material-ui/core/colors/red'

export function createAppTheme() {
	return createMuiTheme({
		typography: {
			fontSize: 14,
			body1: {
				fontSize: '1.5rem',
			},
		},
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
}
