tell application "Numbers"
	activate
	set newDoc to make new document
	
	-- 1. DASHBOARD
	tell sheet 1 of newDoc
		set name to "Treasury Dashboard"
		delete every table
		set dashTable to make new table with properties {name:"Inputs", row count:20, column count:5}
		
		tell dashTable
			set background color of range "A1:E1" to {0, 20000, 50000} -- Deep Blue
			set font name of range "A1:E1" to "Helvetica-Bold"
			set text color of range "A1:E1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "Category"
			set value of cell 2 of row 1 to "Parameter"
			set value of cell 3 of row 1 to "Input"
			set value of cell 4 of row 1 to "Unit"
			set value of cell 5 of row 1 to "Notes"
			
			-- MARKET DATA
			set value of cell 1 of row 2 to "MARKET DATA"
			set font name of cell 1 of row 2 to "Helvetica-Bold"
			set value of cell 2 of row 3 to "SOL Price (Start)"
			set value of cell 3 of row 3 to 150
			set background color of cell 3 of row 3 to {60000, 65535, 60000}
			
			set value of cell 2 of row 4 to "LST APY"
			set value of cell 3 of row 4 to 0.07 -- 7% Yield
			set background color of cell 3 of row 4 to {60000, 65535, 60000}
			set value of cell 5 of row 4 to "Staking Yield"
			
			-- GROWTH
			set value of cell 1 of row 6 to "GROWTH ENGINE"
			set font name of cell 1 of row 6 to "Helvetica-Bold"
			set value of cell 2 of row 7 to "Starting TPS"
			set value of cell 3 of row 7 to 20
			set value of cell 2 of row 8 to "TPS Growth/Mo"
			set value of cell 3 of row 8 to 5
			
			-- REVENUE
			set value of cell 1 of row 10 to "REVENUE"
			set font name of cell 1 of row 10 to "Helvetica-Bold"
			set value of cell 2 of row 11 to "Tx Fee (SOL)"
			set value of cell 3 of row 11 to 0.000005
			set value of cell 2 of row 12 to "Deploy Rev (SOL/Mo)"
			set value of cell 3 of row 12 to 30 -- ~ $4500
			set background color of cell 3 of row 12 to {60000, 65535, 60000}
			
			set value of cell 2 of row 13 to "Capture %"
			set value of cell 3 of row 13 to 1.00 -- 100% to Treasury
			set value of cell 5 of row 13 to "% of Rev to LST"
			
		end tell
	end tell
	
	-- 2. 5-YEAR ACCUMULATION
	tell newDoc
		make new sheet
		set name of last sheet to "Treasury Projection"
		tell sheet "Treasury Projection"
			delete every table
			set simTable to make new table with properties {name:"Timeline", row count:61, column count:7}
			tell simTable
				set background color of range "A1:G1" to {0, 20000, 50000}
				set font name of range "A1:G1" to "Helvetica-Bold"
				set text color of range "A1:G1" to {65535, 65535, 65535}
				
				set value of cell 1 of row 1 to "Month"
				set value of cell 2 of row 1 to "TPS"
				set value of cell 3 of row 1 to "New Revenue (SOL)"
				set value of cell 4 of row 1 to "To Treasury (SOL)"
				set value of cell 5 of row 1 to "Staking Yield (SOL)"
				set value of cell 6 of row 1 to "Total Treasury (SOL)"
				set value of cell 7 of row 1 to "Treasury Value ($)"
				
				-- Row 2 (Month 1)
				set value of cell 1 of row 2 to 1
				set value of cell 2 of row 2 to "=Treasury Dashboard::Inputs::C7" -- Start TPS
				
				-- Revenue (SOL)
				set value of cell 3 of row 2 to "=(B2 * 2592000 * Treasury Dashboard::Inputs::C11) + Treasury Dashboard::Inputs::C12"
				
				-- To Treasury (SOL)
				set value of cell 4 of row 2 to "=C2 * Treasury Dashboard::Inputs::C13"
				
				-- Yield (Month 1 = 0)
				set value of cell 5 of row 2 to 0
				
				-- Total Treasury
				set value of cell 6 of row 2 to "=D2"
				
				-- Value (USD)
				set value of cell 7 of row 2 to "=F2 * Treasury Dashboard::Inputs::C3"
				
				-- LOOP Months 2-60
				repeat with i from 3 to 61
					set prev to i - 1
					set value of cell 1 of row i to (i - 1)
					
					-- TPS Growth
					set value of cell 2 of row i to "=B" & prev & " + Treasury Dashboard::Inputs::C8"
					
					-- Revenue (SOL)
					set value of cell 3 of row i to "=(B" & i & " * 2592000 * Treasury Dashboard::Inputs::C11) + Treasury Dashboard::Inputs::C12"
					
					-- To Treasury
					set value of cell 4 of row i to "=C" & i & " * Treasury Dashboard::Inputs::C13"
					
					-- Yield Calculation: (Previous Balance * APY) / 12
					set value of cell 5 of row i to "=(F" & prev & " * Treasury Dashboard::Inputs::C4) / 12"
					
					-- Total Treasury: PrevBalance + NewDeposit + Yield
					set value of cell 6 of row i to "=F" & prev & " + D" & i & " + E" & i
					
					-- Value (USD)
					set value of cell 7 of row i to "=F" & i & " * Treasury Dashboard::Inputs::C3"
				end repeat
				
				set format of column 3 to number
				set format of column 4 to number
				set format of column 5 to number
				set format of column 6 to number
				set format of column 7 to currency
				
			end tell
		end tell
	end tell
	
	set active sheet of newDoc to sheet 1 of newDoc
end tell
