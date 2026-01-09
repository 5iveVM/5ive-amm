tell application "Numbers"
	activate
	set newDoc to make new document
	
	-- 1. DASHBOARD
	tell sheet 1 of newDoc
		set name to "Tokenomics Dashboard"
		delete every table
		set dashTable to make new table with properties {name:"Inputs", row count:20, column count:5}
		
		tell dashTable
			set background color of range "A1:E1" to {0, 0, 50000}
			set font name of range "A1:E1" to "Helvetica-Bold"
			set text color of range "A1:E1" to {65535, 65535, 65535}
			
			set value of cell 1 of row 1 to "Category"
			set value of cell 2 of row 1 to "Parameter"
			set value of cell 3 of row 1 to "Input"
			set value of cell 4 of row 1 to "Unit"
			set value of cell 5 of row 1 to "Notes"
			
			set value of cell 1 of row 2 to "TOKEN SPECS"
			set font name of cell 1 of row 2 to "Helvetica-Bold"
			set value of cell 2 of row 3 to "Total Supply"
			set value of cell 3 of row 3 to 1000000000
			set background color of cell 3 of row 3 to {60000, 65535, 60000}
			set value of cell 2 of row 4 to "Current Price"
			set value of cell 3 of row 4 to 0.01
			set background color of cell 3 of row 4 to {60000, 65535, 60000}
			
			set value of cell 1 of row 6 to "GROWTH ENGINE"
			set font name of cell 1 of row 6 to "Helvetica-Bold"
			set value of cell 2 of row 7 to "Starting TPS"
			set value of cell 3 of row 7 to 20
			set value of cell 2 of row 8 to "TPS Growth / Month"
			set value of cell 3 of row 8 to 5
			set background color of cell 3 of row 8 to {60000, 65535, 60000}
			
			set value of cell 1 of row 10 to "REVENUE & BURN"
			set font name of cell 1 of row 10 to "Helvetica-Bold"
			set value of cell 2 of row 11 to "Tx Fee (USD)"
			set value of cell 3 of row 11 to 0.00075
			set value of cell 2 of row 12 to "Deploy Rev (Fixed)"
			set value of cell 3 of row 12 to 5000
			set value of cell 2 of row 13 to "Buyback %"
			set value of cell 3 of row 13 to 0.30
			set background color of cell 3 of row 13 to {60000, 65535, 60000}
			
			set value of cell 1 of row 15 to "VALUATION"
			set font name of cell 1 of row 15 to "Helvetica-Bold"
			set value of cell 2 of row 16 to "Utility P/E"
			set value of cell 3 of row 16 to 20
			set value of cell 2 of row 17 to "Hype P/E"
			set value of cell 3 of row 17 to 50
		end tell
	end tell
	
	-- 2. 5-YEAR BURN SIMULATION
	tell newDoc
		make new sheet
		set name of last sheet to "5 Year Projection"
		tell sheet "5 Year Projection"
			delete every table
			set simTable to make new table with properties {name:"Timeline", row count:61, column count:8}
			tell simTable
				set background color of range "A1:H1" to {50000, 0, 0}
				set font name of range "A1:H1" to "Helvetica-Bold"
				set text color of range "A1:H1" to {65535, 65535, 65535}
				
				set value of cell 1 of row 1 to "Month"
				set value of cell 2 of row 1 to "TPS"
				set value of cell 3 of row 1 to "Total Revenue"
				set value of cell 4 of row 1 to "Buy Pressure"
				set value of cell 5 of row 1 to "Tokens Burned"
				set value of cell 6 of row 1 to "Supply Left"
				set value of cell 7 of row 1 to "Price (Utility)"
				set value of cell 8 of row 1 to "Price (Hype)"
				
				-- INITIALIZE MONTH 1 (Hardcoded references to break circularity)
				set value of cell 1 of row 2 to 1
				set value of cell 2 of row 2 to "=Tokenomics Dashboard::Inputs::C7" -- Start TPS
				
				-- Revenue
				set value of cell 3 of row 2 to "=(B2 * 2592000 * Tokenomics Dashboard::Inputs::C11) + Tokenomics Dashboard::Inputs::C12"
				
				-- Buy Pressure
				set value of cell 4 of row 2 to "=C2 * Tokenomics Dashboard::Inputs::C13"
				
				-- Burn (Uses Start Price from Dashboard for Month 1)
				set value of cell 5 of row 2 to "=D2 / Tokenomics Dashboard::Inputs::C4"
				
				-- Supply
				set value of cell 6 of row 2 to "=Tokenomics Dashboard::Inputs::C3 - E2"
				
				-- Utility Price
				set value of cell 7 of row 2 to "=((C2 * 12) * Tokenomics Dashboard::Inputs::C16) / F2"
				
				-- Hype Price
				set value of cell 8 of row 2 to "=((C2 * 12) * Tokenomics Dashboard::Inputs::C17) / F2"
				
				-- LOOP for Months 2-60
				repeat with i from 3 to 61
					set prev to i - 1
					set value of cell 1 of row i to (i - 1)
					
					-- TPS Growth
					set value of cell 2 of row i to "=B" & prev & " + Tokenomics Dashboard::Inputs::C8"
					
					-- Revenue
					set value of cell 3 of row i to "=(B" & i & " * 2592000 * Tokenomics Dashboard::Inputs::C11) + Tokenomics Dashboard::Inputs::C12"
					
					-- Buy Pressure
					set value of cell 4 of row i to "=C" & i & " * Tokenomics Dashboard::Inputs::C13"
					
					-- Tokens Burned (Uses PREVIOUS MONTH'S Utility Price 'G' to estimate burn cost)
					set value of cell 5 of row i to "=D" & i & " / G" & prev
					
					-- Supply Left (Prev Supply - Current Burn)
					set value of cell 6 of row i to "=F" & prev & " - E" & i
					
					-- Utility Price (Annual Rev * PE / Supply)
					set value of cell 7 of row i to "=((C" & i & " * 12) * Tokenomics Dashboard::Inputs::C16) / F" & i
					
					-- Hype Price
					set value of cell 8 of row i to "=((C" & i & " * 12) * Tokenomics Dashboard::Inputs::C17) / F" & i
				end repeat
				
				set format of column 3 to currency
				set format of column 4 to currency
				set format of column 7 to currency
				set format of column 8 to currency
				set format of column 5 to number
				set format of column 6 to number
				
			end tell
		end tell
	end tell
	
	set active sheet of newDoc to sheet 1 of newDoc
end tell
