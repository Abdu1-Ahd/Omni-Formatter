#!/usr/bin/env swift

import Foundation

// ── CASE 1: Struct — mixed spacing ────────────────────────────────────────
struct Point {
    var x :Double
    var y: Double
    var z:Double

    init( x:Double , y:Double , z:Double=0 ) {
        self.x=x; self.y=y; self.z=z
    }

    func distance(to other:Point)->Double {
        let dx=x-other.x; let dy=y-other.y; let dz=z-other.z
        return sqrt(dx*dx + dy*dy + dz*dz)
    }
}

// ── CASE 2: Protocol and extension ────────────────────────────────────────
protocol Describable {
    var description: String { get }
    func describe()
}

extension Describable {
    func describe() {
        print(description)
    }
}

extension Point: Describable {
    var description: String {
        return "Point(\(x), \(y), \(z))"
    }
}

// ── CASE 3: Enum with associated values ────────────────────────────────────
enum Result<T> {
    case success(T)
    case failure(Error)
    case loading

    var isSuccess: Bool {
        if case .success = self { return true }
        return false
    }
}

// ── CASE 4: Generics and where clause ─────────────────────────────────────
func max<T: Comparable>(_ a:T, _ b:T)->T {
    return a > b ? a : b
}

func printAll<T: CustomStringConvertible>(_ items:[T]) where T: Hashable {
    Set(items).forEach { print($0) }
}

// ── CASE 5: Async/await ────────────────────────────────────────────────────
func fetchData(from url:URL) async throws -> Data {
    let (data, response) = try await URLSession.shared.data(from: url)
    guard let httpResponse = response as? HTTPURLResponse,
          httpResponse.statusCode == 200 else {
        throw URLError(.badServerResponse)
    }
    return data
}

// ── CASE 6: Long line ─────────────────────────────────────────────────────
func veryLongFunctionNameThatExceedsLineWidth(parameterOne: String, parameterTwo: Int, parameterThree: Double, parameterFour: Bool = false) -> String {
    return "\(parameterOne) \(parameterTwo) \(parameterThree) \(parameterFour)"
}

// ── CASE 7: Trailing whitespace ───────────────────────────────────────────
func trailingExample() {   
    let x = 1   
    let y = 2  
    print(x + y)
}
