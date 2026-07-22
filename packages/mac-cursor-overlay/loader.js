const nativeBinding = require('./index.js')

const OverlayLevel = Object.freeze({
  Normal: 'normal',
  Floating: 'floating',
  TornOffMenu: 'torn-off-menu',
  ModalPanel: 'modal-panel',
  MainMenu: 'main-menu',
  Status: 'status',
  PopUpMenu: 'pop-up-menu',
  ScreenSaver: 'screen-saver'
})

const levelMap = Object.freeze({
  normal: nativeBinding.OverlayLevel.Normal,
  floating: nativeBinding.OverlayLevel.Floating,
  'torn-off-menu': nativeBinding.OverlayLevel.TornOffMenu,
  'modal-panel': nativeBinding.OverlayLevel.ModalPanel,
  'main-menu': nativeBinding.OverlayLevel.MainMenu,
  status: nativeBinding.OverlayLevel.Status,
  'pop-up-menu': nativeBinding.OverlayLevel.PopUpMenu,
  'screen-saver': nativeBinding.OverlayLevel.ScreenSaver
})

function mapLevel(level) {
  const mapped = levelMap[level]
  if (!mapped) {
    throw new Error(`Unknown overlay level: ${level}`)
  }
  return mapped
}

module.exports = {
  OverlayLevel,
  disposeOverlay: nativeBinding.disposeOverlay,
  flashClick: nativeBinding.flashClick,
  getOverlayState: nativeBinding.getOverlayState,
  hideOverlay: nativeBinding.hideOverlay,
  jumpCursor: nativeBinding.jumpCursor,
  setAttachedWindowId: nativeBinding.setAttachedWindowId,
  setCursorPressed: nativeBinding.setCursorPressed,
  setCursorVisible: nativeBinding.setCursorVisible,
  setDebugBackdrop: nativeBinding.setDebugBackdrop,
  setOverlayBounds: nativeBinding.setOverlayBounds,
  setOverlayLevel(level) {
    return nativeBinding.setOverlayLevel(mapLevel(level))
  },
  showOverlay(bounds, level) {
    return nativeBinding.showOverlay(bounds, mapLevel(level))
  }
}
