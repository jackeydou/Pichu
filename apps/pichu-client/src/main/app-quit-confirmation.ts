export type AppQuitBlockers = {
  runningAgentCount: number
  backgroundTerminalCount: number
}

export type AppQuitDialogLanguage = 'en' | 'zh-CN'

export type AppQuitDialogCopy = {
  message: string
  detail: string
  confirmLabel: string
  cancelLabel: string
}

export function hasAppQuitBlockers(blockers: AppQuitBlockers): boolean {
  return blockers.runningAgentCount > 0 || blockers.backgroundTerminalCount > 0
}

export function appQuitDialogCopy(
  blockers: AppQuitBlockers,
  language: AppQuitDialogLanguage
): AppQuitDialogCopy {
  const hasRunningAgent = blockers.runningAgentCount > 0
  const hasBackgroundTerminals = blockers.backgroundTerminalCount > 0

  if (language === 'zh-CN') {
    if (hasRunningAgent && hasBackgroundTerminals) {
      return {
        message: '退出 Pichu？',
        detail:
          '这台电脑上的当前工作和本地任务会被中断。Pichu 启动的本地网站或服务会在关闭后停止。',
        confirmLabel: '退出',
        cancelLabel: '取消'
      }
    }
    if (hasRunningAgent) {
      return {
        message: '退出 Pichu？',
        detail: '这台电脑上的当前工作会被中断。Pichu 关闭后正在处理的请求不会继续运行。',
        confirmLabel: '退出',
        cancelLabel: '取消'
      }
    }
    return {
      message: '退出 Pichu？',
      detail: '这台电脑上的本地任务会被停止。Pichu 启动的本地网站或服务会在关闭后停止。',
      confirmLabel: '退出',
      cancelLabel: '取消'
    }
  }

  if (hasRunningAgent && hasBackgroundTerminals) {
    return {
      message: 'Quit Pichu?',
      detail:
        'Active local work and tasks on this machine will be interrupted. Any local sites or services Pichu started will stop while Pichu is closed.',
      confirmLabel: 'Quit',
      cancelLabel: 'Cancel'
    }
  }
  if (hasRunningAgent) {
    return {
      message: 'Quit Pichu?',
      detail:
        'Active local work on this machine will be interrupted and running requests will not continue while Pichu is closed.',
      confirmLabel: 'Quit',
      cancelLabel: 'Cancel'
    }
  }
  return {
    message: 'Quit Pichu?',
    detail:
      'Active local tasks on this machine will be stopped. Any local sites or services Pichu started will stop while Pichu is closed.',
    confirmLabel: 'Quit',
    cancelLabel: 'Cancel'
  }
}
