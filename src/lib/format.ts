const numberFormatter = new Intl.NumberFormat('zh-CN')

export const formatNumber = (value: number) => numberFormatter.format(value)
