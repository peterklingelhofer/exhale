// CustomMenu.swift
import Cocoa

class CustomMenu: NSMenu {
    weak var appDelegate: AppDelegate?
    
    required init(coder: NSCoder) {
        super.init(coder: coder)
    }
    
    override init(title: String) {
        super.init(title: title)
    }
    
    @objc func showSettings(_ sender: AnyObject?) {
        appDelegate?.toggleSettings(sender)
    }
    
    @objc func quitApp(_ sender: AnyObject?) {
        NSApp.terminate(sender)
    }
}
