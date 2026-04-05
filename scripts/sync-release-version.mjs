import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const projectRoot = path.resolve(__dirname, '..')

const packageJsonPath = path.join(projectRoot, 'package.json')
const tauriConfigPath = path.join(projectRoot, 'src-tauri', 'tauri.conf.json')
const cargoTomlPath = path.join(projectRoot, 'src-tauri', 'Cargo.toml')

async function readPackageVersion() {
  const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf8'))
  return packageJson.version
}

async function syncPackageJson(version) {
  const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf8'))
  if (packageJson.version === version) {
    return
  }

  packageJson.version = version
  await writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)
}

async function syncTauriConfig(version) {
  const tauriConfig = JSON.parse(await readFile(tauriConfigPath, 'utf8'))
  if (tauriConfig.version === version) {
    return
  }

  tauriConfig.version = version
  await writeFile(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`)
}

async function syncCargoToml(version) {
  const cargoToml = await readFile(cargoTomlPath, 'utf8')
  const versionPattern = /^version = "[^"]+"$/m
  if (!versionPattern.test(cargoToml)) {
    throw new Error('Failed to update version in src-tauri/Cargo.toml')
  }

  const nextCargoToml = cargoToml.replace(versionPattern, `version = "${version}"`)

  if (nextCargoToml === cargoToml) {
    return
  }

  await writeFile(cargoTomlPath, nextCargoToml)
}

async function main() {
  const cliVersion = process.argv[2]
  const version = cliVersion ?? await readPackageVersion()

  await syncPackageJson(version)
  await syncTauriConfig(version)
  await syncCargoToml(version)

  console.log(`Synced release version to ${version}`)
}

await main()
