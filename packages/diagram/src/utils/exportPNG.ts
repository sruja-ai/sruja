export async function exportPNGFromSVG(svg: string, scale = 2): Promise<string> {
  const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const img = await new Promise<HTMLImageElement>((resolve, reject) => {
    const i = new Image()
    i.onload = () => resolve(i)
    i.onerror = (e) => reject(e)
    i.src = url
  })
  const w = img.width * scale
  const h = img.height * scale
  const canvas = document.createElement('canvas')
  canvas.width = w
  canvas.height = h
  const ctx = canvas.getContext('2d')!
  ctx.drawImage(img, 0, 0, w, h)
  URL.revokeObjectURL(url)
  return canvas.toDataURL('image/png')
}

