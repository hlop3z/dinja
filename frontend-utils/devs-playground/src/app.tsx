import { useState } from 'preact/hooks'
import './app.css'
import engine from './tests/core'
import Component from './tests/comp'

console.log(engine.render(Component));


export function App() {

  return (
    <>
      <h1>Hello World</h1>
    </>
  )
}
