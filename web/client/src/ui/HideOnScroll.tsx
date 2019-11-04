import React from 'react'
import { Slide, useScrollTrigger } from '@material-ui/core'

interface Props {
	target?: Node
	children: React.ReactElement
}

const HideOnScroll: React.FC<Props> = ({ children, target }) => {
	const trigger = useScrollTrigger({ target })
	return (
		<Slide appear={false} direction="down" in={!trigger}>
			{children}
		</Slide>
	)
}

export default HideOnScroll
