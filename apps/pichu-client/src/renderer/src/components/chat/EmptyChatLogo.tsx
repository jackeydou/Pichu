import { useI18n } from '@renderer/lib/i18n'
import { useState } from 'react'
import pichuHomeMark from '../../../../../resources/pichu-home-mark.png?asset'

export function EmptyChatLogo(): React.JSX.Element {
  const { t } = useI18n()
  const [animationKey, setAnimationKey] = useState(0)

  return (
    <button
      type="button"
      aria-label={t('chat.logo.animate')}
      className="pichu-empty-chat-logo mx-auto block w-44 select-none sm:w-48"
      onClick={() => setAnimationKey((currentKey) => currentKey + 1)}
    >
      <span
        key={animationKey}
        aria-hidden="true"
        className={`pichu-empty-chat-logo-art block ${animationKey > 0 ? 'is-animated' : ''}`}
      >
        <img
          alt=""
          className="pichu-empty-chat-ear pichu-empty-chat-ear-left"
          draggable={false}
          src={pichuHomeMark}
        />
        <img
          alt=""
          className="pichu-empty-chat-ear pichu-empty-chat-ear-right"
          draggable={false}
          src={pichuHomeMark}
        />
      </span>
    </button>
  )
}
