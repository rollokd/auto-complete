// Generates Auto's app and menubar icons: swift scripts/generate-icons.swift <out-dir>
import AppKit

let outDir = URL(fileURLWithPath: CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : ".")

func render(size: Int, draw: (CGContext, CGFloat) -> Void) -> Data {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
        samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
        bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    draw(NSGraphicsContext.current!.cgContext, CGFloat(size))
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])!
}

func drawGlyph(_ s: CGFloat, color: NSColor, scale: CGFloat) {
    let font = NSFont.monospacedSystemFont(ofSize: s * scale, weight: .heavy)
    let text = NSAttributedString(string: "›_", attributes: [.font: font, .foregroundColor: color])
    let bounds = text.size()
    text.draw(at: CGPoint(x: (s - bounds.width) / 2, y: (s - bounds.height) / 2 + s * 0.02))
}

func appIcon(_ size: Int) -> Data {
    render(size: size) { ctx, s in
        // macOS icon grid: ~10% inset, continuous-ish corner radius
        let inset = s * 0.1
        let rect = CGRect(x: inset, y: inset, width: s - inset * 2, height: s - inset * 2)
        let path = CGPath(roundedRect: rect, cornerWidth: rect.width * 0.225, cornerHeight: rect.width * 0.225, transform: nil)
        ctx.addPath(path)
        ctx.clip()
        let colors = [NSColor(red: 0.42, green: 0.25, blue: 0.85, alpha: 1).cgColor,
                      NSColor(red: 0.16, green: 0.10, blue: 0.38, alpha: 1).cgColor] as CFArray
        let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(), colors: colors, locations: [0, 1])!
        ctx.drawLinearGradient(gradient, start: CGPoint(x: 0, y: s), end: CGPoint(x: 0, y: 0), options: [])
        drawGlyph(s, color: .white, scale: 0.36)
    }
}

func trayIcon(_ size: Int, color: NSColor) -> Data {
    render(size: size) { _, s in drawGlyph(s, color: color, scale: 0.75) }
}

func write(_ data: Data, _ name: String, in dir: URL = outDir) {
    try! data.write(to: dir.appendingPathComponent(name))
}

write(appIcon(512), "icon.png")
write(appIcon(32), "32x32.png")
write(appIcon(128), "128x128.png")
write(appIcon(256), "128x128@2x.png")
write(trayIcon(36, color: .black), "icon-monochrome.png")
write(trayIcon(36, color: .white), "icon-monochrome-light.png")

let iconset = outDir.appendingPathComponent("icon.iconset")
try? FileManager.default.createDirectory(at: iconset, withIntermediateDirectories: true)
for base in [16, 32, 128, 256, 512] {
    write(appIcon(base), "icon_\(base)x\(base).png", in: iconset)
    write(appIcon(base * 2), "icon_\(base)x\(base)@2x.png", in: iconset)
}
