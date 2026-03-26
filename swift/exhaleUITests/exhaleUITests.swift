//
//  exhaleUITests.swift
//  exhaleUITests
//
//  Created by Peter Klingelhofer on 3/15/23.
//

import XCTest

class exhaleUITests: XCTestCase {

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {}

    func testAppLaunches() throws {
        let app = XCUIApplication()
        app.launch()
        XCTAssertTrue(app.exists, "App should launch successfully")
    }
}
