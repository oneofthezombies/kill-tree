import { spawn } from 'child_process'
spawn('node', ['../tests/resources/sleep.mjs'], {
    stdio: 'inherit',
})
setTimeout(() => {}, 5000)
