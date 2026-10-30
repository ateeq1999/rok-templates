// Ensure jsdom globals are available before tests run
import { JSDOM } from "jsdom"

const dom = new JSDOM("<!DOCTYPE html><html><body></body></html>", {
  url: "http://localhost",
  pretendToBeVisual: true,
})

const win = dom.window as unknown as Window & typeof globalThis
global.window = win
global.document = win.document
global.navigator = win.navigator
global.HTMLElement = win.HTMLElement
global.Element = win.Element
global.Node = win.Node
global.requestAnimationFrame = (cb) => setTimeout(cb, 16)
global.cancelAnimationFrame = (id) => clearTimeout(id)