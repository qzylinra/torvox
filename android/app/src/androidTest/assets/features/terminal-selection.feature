@REQ_SEL_001 @REQ_SEL_002
Feature: Text Selection
  The terminal supports long-press text selection with
  selection handles and a context menu.

  @REQ_SEL_001
  Scenario: Long press blank area shows paste popup
    Given a terminal session is active
    When the user long-presses a blank cell
    Then the cell is highlighted with inverted colors
    And a paste popup is shown nearby

  @REQ_SEL_001
  Scenario: Long press text highlights the word
    Given a terminal session is active with visible text
    When the user long-presses on a word
    Then the word becomes selected with inverted colors
    And selection handles appear at the start and end

  @REQ_SEL_001
  Scenario: Selection handles adjust selected region
    Given text is selected by long-press
    When the user drags the start handle to the left
    Then the selected region expands
    And the highlighted area updates accordingly

  @REQ_SEL_002
  Scenario: Context menu appears after selection
    Given text is selected
    Then a context menu with "Copy" and "Select All" buttons is displayed
    And the menu does not overlap the selected text

  @REQ_SEL_002
  Scenario: Copy copies selected text to clipboard
    Given text is selected
    When the user taps "Copy"
    Then the selected text is copied to clipboard
    And the selection is cleared

  @REQ_SEL_002
  Scenario: Select All selects entire screen content
    Given a terminal session is active with visible text
    When the user selects all text
    Then all text on screen is selected with inverted colors

  @REQ_SEL_002
  Scenario: Session drawer closes during selection
    Given the session drawer is open
    When the user starts selecting text on the terminal
    Then the drawer closes
    And the terminal is focused for interaction
