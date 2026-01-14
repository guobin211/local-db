import 'dotenv/config'
import { execa } from 'execa'

async function build() {
  try {
    console.log('开始构建 Tauri 应用...')

    await execa('pnpm', ['tauri', 'build'], {
      stdio: 'inherit',
      env: process.env
    })

    console.log('构建完成!')
  } catch (error) {
    console.error('构建失败:', error.message)
    process.exit(1)
  }
}

build()
