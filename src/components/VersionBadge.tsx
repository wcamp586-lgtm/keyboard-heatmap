import { useEffect, useState } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { nativeBridgeAvailable } from '../lib/tauri'

export function VersionBadge() {
  const [version, setVersion] = useState<string | null>(() => nativeBridgeAvailable() ? null : __APP_VERSION__)
  const [failed, setFailed] = useState(false)
  useEffect(() => {
    if (!nativeBridgeAvailable()) { document.title = `键盘热力 v${__APP_VERSION__} · 浏览器演示`; return }
    let active = true
    getVersion().then((value) => {
      if (active) { setVersion(value); document.title = `键盘热力 v${value}` }
    }).catch(() => { if (active) setFailed(true) })
    return () => { active = false }
  }, [])
  return <span className="version-badge" aria-label="当前运行版本" title="当前运行程序的版本；不进行联网更新检查">{failed ? '版本读取失败' : version ? `v${version}` : '版本读取中'}</span>
}
