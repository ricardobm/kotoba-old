import * as express from 'express'

const PORT     = 8080
const APP_NAME = 'Hongo'

const app  = express()

app.get('/', (req, res) => {
    res.send(`Hello from ${APP_NAME}`)
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
