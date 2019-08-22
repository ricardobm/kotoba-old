import * as express from 'express'
import * as bodyParser from 'body-parser'
import * as fs from 'fs'
import * as path from 'path'

import * as dict from './dict'

const PORT     = 3001
const APP_NAME = 'Hongo'
const DATA_DIR = 'hongo-data'

// Search the file system for the user data directory, moving up the directory
// hierarchy.
//
// TODO: use a configuration instead
const userData = (() => {
    let lastPath = ''
    let curPath = path.resolve('.')
    while (curPath !== lastPath) {
        if (fs.existsSync(path.join(curPath, DATA_DIR, 'import'))) {
            return path.resolve(path.join(curPath, DATA_DIR))
        }
        lastPath = curPath
        curPath = path.resolve(path.join(curPath, '..'))
    }
    console.error('Error: could not find data directory!')
    process.exit(1)
})()!

main().catch(err => console.error('Error:', err))

async function main() {
    console.log(`Found data directory at ${userData}`)

    const start = process.hrtime()
    await loadDicts()
    const end = process.hrtime(start)
    console.log(`Loading took ${(end[0] + end[1] / 1e9).toFixed(3)}s`)

    // Create the Express application
    const app = express()
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
}

async function loadDicts() {
    const baseDir = path.join(userData, 'import')
    const dicts = await Promise.all(
        fs.readdirSync(baseDir)
            .map(it => path.join(baseDir, it))
            .map(dict.importDict)
    )
    console.log(dicts.map(
        it => `==> ${it.title} (${it.revision})\n    In ${it.path}\n    Banks: ${it.banks()}`).join('\n'))

    const count: {[key: string]: number} = dicts.reduce((acc, it) => it.count(acc), null)!
    const loaded = Object.keys(count).map(key => `${key}:${count[key]}`).join(' / ')
    console.log(`\nLoaded ${loaded}`)

    const allRules: {[key: string]: boolean} = {}
    const ruleSample: {[key: string]: string[]} = {}

    const tagMap: {[key: string]: dict.ImportedTag} = {}
    const termTags: {[key:string]: boolean} = {}
    const definitionTags: {[key:string]: boolean} = {}
    const kanjiTags: {[key:string]: boolean} = {}
    for (const dict of dicts) {
        for (const tag of dict.tags) {
            tagMap[tag.name] = tag
        }
        for (const term of dict.terms) {
            for (const tag of term.termTags) {
                termTags[tag] = true
            }
            for (const tag of term.definitionTags) {
                definitionTags[tag] = true
            }
            for (const rule of term.rules) {
                allRules[rule] = true
                ruleSample[rule] = ruleSample[rule] || []
                if (ruleSample[rule].length < 10) {
                    if (Math.random() < 0.1) {
                        ruleSample[rule].push(term.expression)
                    }
                }
            }
        }
        for (const term of dict.kanjis) {
            for (const tag of term.tags) {
                kanjiTags[tag] = true
            }
        }
    }

    function showTags(name: string, m: {[key:string]: boolean}) {
        const notFound = Object.keys(m).filter(k => !tagMap[k])
        console.log(`\n${name} tags:`)
        console.log(
            Object.keys(m).sort()
                .filter(k => tagMap[k])
                .map(k => tagMap[k])
                .map(x => `- ${x.name} (${x.category}) ${x.notes}`)
                .join('\n'))
        if (notFound.length) {
            console.error(`\nWARNING: tags not defined in ${name}: ${notFound.join(', ')}`)
        }
    }

    showTags('Term', termTags)
    showTags('Definition', definitionTags)
    showTags('Kanji', kanjiTags)

    console.log(`\nRULES: ${Object.keys(allRules).sort().join(', ')}\n`)
    for (const key of Object.keys(allRules).sort()) {
        console.log(`- ${key}: ${ruleSample[key].sort().join(', ')}`)
    }
}
