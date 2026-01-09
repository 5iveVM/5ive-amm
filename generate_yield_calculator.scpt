tell application "Numbers"
	activate
	set newDoc to make new document
	
	tell sheet 1 of newDoc
		set name to "Yield Dashboard"
		delete every table
		
		-- TABLE 1: INPUTS
		set inputTable to make new table with properties {name:"Inputs", row count:15, column count:5}
		tell inputTable
			set background color of range "A1:E1" to {0, 0, 0}
			set font name of range "A1:E1" to "Helvetica-Bold"
			set text color of range "A1:E1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "Category"
			set value of cell 2 of row 1 to "Parameter"
			set value of cell 3 of row 1 to "Input"
			set value of cell 4 of row 1 to "Unit"
			set value of cell 5 of row 1 to "Notes"
			
			set value of cell 1 of row 2 to "GLOBAL"
			set value of cell 2 of row 2 to "SOL Price"
			set value of cell 3 of row 2 to 150
			set background color of cell 3 of row 2 to {60000, 65535, 60000}
			set value of cell 4 of row 2 to "USD"
			
			set value of cell 2 of row 3 to "LST APY"
			set value of cell 3 of row 3 to 0.07
			set background color of cell 3 of row 3 to {60000, 65535, 60000}
			set value of cell 5 of row 3 to "Staking Yield"
			
			set value of cell 1 of row 5 to "FEE SCHEDULE"
			set font name of cell 1 of row 5 to "Helvetica-Bold"
			
			set value of cell 2 of row 5 to "Execution Fee (SOL)"
			set value of cell 3 of row 5 to 0.000005
			set background color of cell 3 of row 5 to {60000, 65535, 60000}
			
			set value of cell 2 of row 6 to "Execution Fee (USD)"
			set value of cell 3 of row 6 to "=C5 * C2"
			set value of cell 4 of row 6 to "USD/Tx"
			set font name of cell 3 of row 6 to "Helvetica-Bold"
			
			set value of cell 2 of row 7 to "Avg Deploy Fee (SOL)"
			set value of cell 3 of row 7 to 0.15
			set background color of cell 3 of row 7 to {60000, 65535, 60000}
			
			set value of cell 2 of row 8 to "Avg Deploy Fee (USD)"
			set value of cell 3 of row 8 to "=C7 * C2"
			set value of cell 4 of row 8 to "USD/Deploy"
			
			set value of cell 1 of row 10 to "THROUGHPUT"
			set font name of cell 1 of row 10 to "Helvetica-Bold"
			set value of cell 2 of row 10 to "Average TPS"
			set value of cell 3 of row 10 to 20
			set background color of cell 3 of row 10 to {60000, 65535, 60000}
			set value of cell 4 of row 10 to "Tx/Sec"
			
			set value of cell 2 of row 11 to "Average TPD"
			set value of cell 3 of row 11 to "=C10 * 86400"
			set value of cell 4 of row 11 to "Tx/Day"
			
			set value of cell 2 of row 13 to "Daily Deploys"
			set value of cell 3 of row 13 to 10
			set background color of cell 3 of row 13 to {60000, 65535, 60000}
			set value of cell 4 of row 13 to "Count/Day"
		end tell
		
		-- TABLE 2: YIELD
		set yieldTable to make new table with properties {name:"Protocol Yield", row count:7, column count:5}
		tell yieldTable
			set background color of range "A1:E1" to {50000, 50000, 50000}
			set font name of range "A1:E1" to "Helvetica-Bold"
			set text color of range "A1:E1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "Timeframe"
			set value of cell 2 of row 1 to "Total Tx"
			set value of cell 3 of row 1 to "Total SOL Rev"
			set value of cell 4 of row 1 to "Value (USD)"
			set value of cell 5 of row 1 to "LST Bonus (SOL)"
			
			-- 1 Hour
			set value of cell 1 of row 2 to "1 Hour"
			set value of cell 2 of row 2 to "=Inputs::C10 * 3600"
			set value of cell 3 of row 2 to "=(B2 * Inputs::C5) + ((Inputs::C13 / 24) * Inputs::C7)"
			set value of cell 4 of row 2 to "=C2 * Inputs::C2"
			set value of cell 5 of row 2 to "=C2 * (Inputs::C3 / 8760)" -- Hourly Yield
			
			-- 24 Hours
			set value of cell 1 of row 3 to "24 Hours (Daily)"
			set value of cell 2 of row 3 to "=Inputs::C11"
			set value of cell 3 of row 3 to "=(B3 * Inputs::C5) + (Inputs::C13 * Inputs::C7)"
			set value of cell 4 of row 3 to "=C3 * Inputs::C2"
			set value of cell 5 of row 3 to "=C3 * (Inputs::C3 / 365)" -- Daily Yield
			set background color of row 3 to {60000, 65535, 60000}
			
			-- 7 Days
			set value of cell 1 of row 4 to "7 Days (Weekly)"
			set value of cell 2 of row 4 to "=B3 * 7"
			set value of cell 3 of row 4 to "=C3 * 7"
			set value of cell 4 of row 4 to "=C4 * Inputs::C2"
			set value of cell 5 of row 4 to "=C4 * (Inputs::C3 / 52)" -- Weekly Yield
			
			-- 30 Days
			set value of cell 1 of row 5 to "30 Days (Monthly)"
			set value of cell 2 of row 5 to "=B3 * 30"
			set value of cell 3 of row 5 to "=C3 * 30"
			set value of cell 4 of row 5 to "=C5 * Inputs::C2"
			set value of cell 5 of row 5 to "=C5 * (Inputs::C3 / 12)" -- Monthly Yield
			
			-- 1 Year
			set value of cell 1 of row 6 to "1 Year (Annual)"
			set value of cell 2 of row 6 to "=B3 * 365"
			set value of cell 3 of row 6 to "=C3 * 365"
			set value of cell 4 of row 6 to "=C6 * Inputs::C2"
			set value of cell 5 of row 6 to "=C6 * Inputs::C3" -- Full Annual Yield
			
			set format of column 3 to number
			set format of column 4 to currency
			set format of column 5 to number
		end tell
	end tell
	
	set active sheet of newDoc to sheet 1 of newDoc
end tell
