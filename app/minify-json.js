import fs from 'fs'
import path from 'path'

const assetsDir = path.resolve(import.meta.dirname, 'dist/assets')

const minifyJson = dir => {
  fs.readdirSync(dir).forEach(file => {
    const filePath = path.join(dir, file)
    const stat = fs.statSync(filePath)

    if (stat.isDirectory()) {
      minifyJson(filePath)
    } else if (path.extname(file) === '.json') {
      const content = fs.readFileSync(filePath, 'utf8')
      const minifiedContent = JSON.stringify(JSON.parse(content))
      fs.writeFileSync(filePath, minifiedContent)
      console.log(`Minified: ${filePath}`)
    }
  })
}

minifyJson(assetsDir)
