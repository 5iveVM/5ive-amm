tell application "Numbers"
	activate
	set newDoc to make new document
	
	-- 1. MISSION CONTROL (The HUD)
	tell sheet 1 of newDoc
		set name to "Mission Control"
		delete every table
		
		-- TABLE 1: THE SCOREBOARD
		set scoreTable to make new table with properties {name:"Scoreboard", row count:8, column count:4}
		tell scoreTable
			set background color of range "A1:D1" to {0, 0, 0}
			set font name of range "A1:D1" to "Helvetica-Bold"
			set text color of range "A1:D1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "METRIC"
			set value of cell 2 of row 1 to "GOAL"
			set value of cell 3 of row 1 to "ACTUAL"
			set value of cell 4 of row 1 to "GAP"
			
			-- REVENUE GOAL
			set value of cell 1 of row 2 to "Total Revenue Goal"
			set value of cell 2 of row 2 to 2500
			set value of cell 3 of row 2 to 0 -- INPUT: Bank Balance
			set background color of cell 3 of row 2 to {60000, 65535, 60000}
			set value of cell 4 of row 2 to "=B2 - C2"
			
			-- TIME
			set value of cell 1 of row 3 to "Days Remaining"
			set value of cell 2 of row 3 to 23
			set value of cell 3 of row 3 to 23 -- INPUT: Update Daily
			set background color of cell 3 of row 3 to {65535, 65535, 50000}
			set value of cell 4 of row 3 to "=C3"
			
			-- TRANSACTION VOLUME (The Flywheel)
			set value of cell 1 of row 5 to "TRANSACTION ENGINE"
			set font name of cell 1 of row 5 to "Helvetica-Bold"
			set background color of range "A5:D5" to {0, 20000, 50000}
			set text color of range "A5:D5" to {65535, 65535, 65535}
			
			set value of cell 1 of row 6 to "Current Tx / Day"
			set value of cell 3 of row 6 to 1000 -- INPUT: How many Tx yesterday?
			set background color of cell 3 of row 6 to {60000, 65535, 60000}
			
			set value of cell 1 of row 7 to "Tx Revenue / Day"
			set value of cell 2 of row 7 to 0.000005 * 150 -- Fee ($)
			set value of cell 4 of row 7 to "=C6 * B7" -- Daily Passive Income
			
			-- DAILY QUOTA (The "Gap" minus "Passive Income")
			set value of cell 1 of row 8 to "NET DAILY NEEDED ($)"
			set value of cell 4 of row 8 to "=(D2 / C3) - D7"
			set background color of cell 4 of row 8 to {0, 0, 0}
			set text color of cell 4 of row 8 to {0, 65535, 0}
			set font name of cell 4 of row 8 to "Helvetica-Bold"
			set font size of cell 4 of row 8 to 16
		end tell
		
		-- TABLE 2: DAILY ORDERS
		set actionTable to make new table with properties {name:"Daily Orders", row count:6, column count:2}
		tell actionTable
			set background color of range "A1:B1" to {50000, 0, 0}
			set font name of range "A1:B1" to "Helvetica-Bold"
			set text color of range "A1:B1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "TARGET TYPE"
			set value of cell 2 of row 1 to "QTY NEEDED TODAY"
			
			-- Profit Assumptions: Small=$7.50, Med=$18, Large=$45
			
			set value of cell 1 of row 2 to "Small (Counters)"
			set value of cell 2 of row 2 to "=Scoreboard::D8 / 7.50"
			
			set value of cell 1 of row 3 to "Medium (Tokens)"
			set value of cell 2 of row 3 to "=Scoreboard::D8 / 18.00"
			
			set value of cell 1 of row 4 to "Large (Games)"
			set value of cell 2 of row 4 to "=Scoreboard::D8 / 45.00"
			
			set value of cell 1 of row 6 to "HYBRID MIX"
			set value of cell 2 of row 6 to "=(Scoreboard::D8 * 0.8 / 7.50) + (Scoreboard::D8 * 0.2 / 18.00)"
			set font name of row 6 to "Helvetica-Bold"
		end tell
	end tell
	
	-- 2. DAILY HUSTLE (Log)
	tell newDoc
		make new sheet
		set name of last sheet to "Daily Hustle"
		tell sheet "Daily Hustle"
			delete every table
			set logTable to make new table with properties {name:"Execution Log", row count:25, column count:7}
			tell logTable
				set background color of range "A1:G1" to {20000, 20000, 20000}
				set font name of range "A1:G1" to "Helvetica-Bold"
				set text color of range "A1:G1" to {65535, 65535, 65535}
				
				set value of cell 1 of row 1 to "Date"
				set value of cell 2 of row 1 to "Small Deploys"
				set value of cell 3 of row 1 to "Med Deploys"
				set value of cell 4 of row 1 to "Large Deploys"
				set value of cell 5 of row 1 to "Tx Volume"
				set value of cell 6 of row 1 to "Total Revenue ($)"
				set value of cell 7 of row 1 to "Status"
				
				-- Date Fill
				set value of cell 1 of row 2 to "Jan 8"
				set value of cell 1 of row 3 to "Jan 9"
				
				-- Input Columns (Green)
				set background color of range "B2:E25" to {60000, 65535, 60000}
				
				-- Revenue Formula: (Small*7.5 + Med*18 + Large*45) + (TxVol * Fee)
				set value of cell 6 of row 2 to "=((B2*7.5)+(C2*18)+(D2*45)) + (E2 * 'Mission Control'::Scoreboard::B7)"
			
				-- Status Check
				set value of cell 7 of row 2 to "=IF(F2 >= 'Mission Control'::Scoreboard::D8, \"WIN\", \"FAIL\")"
				
			end tell
		end tell
	end tell

	set active sheet of newDoc to sheet 1 of newDoc
end tell
