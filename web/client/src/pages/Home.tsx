import React, { useEffect, useRef } from 'react'

import * as app from '../store/app'
import * as home from '../store/home'
import { Dispatch, Action } from 'redux'
import { Typography, Button, Link } from '@material-ui/core'
import LinkTo from '../util/LinkTo'
import { AppState } from '../store'
import { connect } from 'react-redux'

interface IDispatch {
	dispatch: Dispatch<Action>
}

interface IState extends home.State {}

interface IProps extends IDispatch, IState {}

const Home: React.FC<IProps> = self => {
	const dispatch = useRef(self.dispatch)
	useEffect(() => {
		dispatch.current(app.setTitle('Home'))
	}, [])

	return (
		<div>
			<Typography variant="body1" component="div">
				<Typography variant="h2" gutterBottom>
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
					Webテキストの日本語に含まれる約物を半角にすることができました。 例えば「かっこ」や『二重かっこ』、
					【バッジに使いそうなかっこ】などを半角にできます。ウェイトは7種類。Noto Sans
					Japaneseに沿っています。
				</p>
				<p className="japanese" style={{ fontSize: '0.3em' }}>
					「約物半角専用のWebフォント」を優先的に当てることによって、
					Webテキストの日本語に含まれる約物を半角にすることができました。 例えば「かっこ」や『二重かっこ』、
					【バッジに使いそうなかっこ】などを半角にできます。ウェイトは7種類。Noto Sans
					Japaneseに沿っています。
				</p>
				<p>
					Edit <code>src/App.tsx</code> and save to reload.
				</p>
				<Link href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
					Learn React
				</Link>
				<LinkTo to="/todo">TODO</LinkTo>
				<LinkTo to="/search">Search</LinkTo>
				<LinkTo to="/ping_pong">Ping pong</LinkTo>
				<p>
					Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore
					et dolore magna aliqua. Rhoncus dolor purus non enim praesent elementum facilisis leo vel. Risus at
					ultrices mi tempus imperdiet. Semper risus in hendrerit gravida rutrum quisque non tellus. Convallis
					convallis tellus id interdum velit laoreet id donec ultrices. Odio morbi quis commodo odio aenean
					sed adipiscing. Amet nisl suscipit adipiscing bibendum est ultricies integer quis. Cursus euismod
					quis viverra nibh cras. Metus vulputate eu scelerisque felis imperdiet proin fermentum leo. Mauris
					commodo quis imperdiet massa tincidunt. Cras tincidunt lobortis feugiat vivamus at augue. At augue
					eget arcu dictum varius duis at consectetur lorem. Velit sed ullamcorper morbi tincidunt. Lorem
					donec massa sapien faucibus et molestie ac. Consequat mauris nunc congue nisi vitae suscipit.
					Fringilla est ullamcorper eget nulla facilisi etiam dignissim diam. Pulvinar elementum integer enim
					neque volutpat ac tincidunt. Ornare suspendisse sed nisi lacus sed viverra tellus. Purus sit amet
					volutpat consequat mauris. Elementum eu facilisis sed odio morbi. Euismod lacinia at quis risus sed
					vulputate odio. Morbi tincidunt ornare massa eget egestas purus viverra accumsan in. In hendrerit
					gravida rutrum quisque non tellus orci ac. Pellentesque nec nam aliquam sem et tortor. Habitant
					morbi tristique senectus et. Adipiscing elit duis tristique sollicitudin nibh sit. Ornare aenean
					euismod elementum nisi quis eleifend. Commodo viverra maecenas accumsan lacus vel facilisis. Nulla
					posuere sollicitudin aliquam ultrices sagittis orci a.
				</p>
				<p>
					Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore
					et dolore magna aliqua. Rhoncus dolor purus non enim praesent elementum facilisis leo vel. Risus at
					ultrices mi tempus imperdiet. Semper risus in hendrerit gravida rutrum quisque non tellus. Convallis
					convallis tellus id interdum velit laoreet id donec ultrices. Odio morbi quis commodo odio aenean
					sed adipiscing. Amet nisl suscipit adipiscing bibendum est ultricies integer quis. Cursus euismod
					quis viverra nibh cras. Metus vulputate eu scelerisque felis imperdiet proin fermentum leo. Mauris
					commodo quis imperdiet massa tincidunt. Cras tincidunt lobortis feugiat vivamus at augue. At augue
					eget arcu dictum varius duis at consectetur lorem. Velit sed ullamcorper morbi tincidunt. Lorem
					donec massa sapien faucibus et molestie ac. Consequat mauris nunc congue nisi vitae suscipit.
					Fringilla est ullamcorper eget nulla facilisi etiam dignissim diam. Pulvinar elementum integer enim
					neque volutpat ac tincidunt. Ornare suspendisse sed nisi lacus sed viverra tellus. Purus sit amet
					volutpat consequat mauris. Elementum eu facilisis sed odio morbi. Euismod lacinia at quis risus sed
					vulputate odio. Morbi tincidunt ornare massa eget egestas purus viverra accumsan in. In hendrerit
					gravida rutrum quisque non tellus orci ac. Pellentesque nec nam aliquam sem et tortor. Habitant
					morbi tristique senectus et. Adipiscing elit duis tristique sollicitudin nibh sit. Ornare aenean
					euismod elementum nisi quis eleifend. Commodo viverra maecenas accumsan lacus vel facilisis. Nulla
					posuere sollicitudin aliquam ultrices sagittis orci a.
				</p>
			</Typography>
		</div>
	)
}

const mapStateToProps = (state: AppState): IState => ({
	label: state.ping_pong.ping ? 'PING' : 'PONG',
	running: state.ping_pong.ping != null,
})

const mapDispatchToProps = (dispatch: Dispatch<Action>): IDispatch => ({
	dispatch,
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(Home)
