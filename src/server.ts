import * as express from 'express'
import * as bodyParser from 'body-parser'

const PORT     = 3001
const APP_NAME = 'Hongo'

const app  = express()

app.use(bodyParser.json())
app.use(bodyParser.urlencoded({ extended: true }))

app.get('/', (req, res) => {
    res.send(`Hello from ${APP_NAME}`)
})

app.get('/list', (req, res) => {
    res.status(200).json({
        items: [
            { id: 'A', text: 'Item A' },
            { id: 'B', text: 'Item B' },
            { id: 'C', text: 'Item C' },
        ]
    })
})

const server = app.listen(PORT, () => {
    console.log(`${APP_NAME} server started at http://localhost:${PORT}`)
})

process.once('SIGINT', shutdown)

function shutdown() {
    console.log('Shutting down...')
    server.close(() => {
        process.exit(0)
    })
}
