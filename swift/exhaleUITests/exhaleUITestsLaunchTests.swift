//
//  exhaleUITestsLaunchTests.swift
//  exhaleUITests
//
//  Created by Peter Klingelhofer on 3/15/23.
//

import XCTest

class exhaleUITestsLaunchTests: XCTestCase {

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testLaunchAndScreenshot() throws {
        let app = XCUIApplication()
        app.launch()

        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = "Launch Screen"
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
