tell application "Numbers"
	activate
	set newDoc to make new document
	
	-- 1. THE DASHBOARD (Command Center)
	tell sheet 1 of newDoc
		set name to "Dashboard"
		delete every table
		set dashTable to make new table with properties {name:"Inputs", row count:22, column count:5}
		tell dashTable
			set background color of range "A1:E1" to {50000, 50000, 55000}
			set font name of range "A1:E1" to "Helvetica-Bold"
			set value of cell 1 of row 1 to "Category"
			set value of cell 2 of row 1 to "Parameter"
			set value of cell 3 of row 1 to "Input (SOL)"
			set value of cell 4 of row 1 to "Input (USD)"
			set value of cell 5 of row 1 to "Notes"
			
			set value of cell 1 of row 2 to "GLOBAL INPUTS"
			set font name of cell 1 of row 2 to "Helvetica-Bold"
			set value of cell 2 of row 3 to "SOL Price"
			set value of cell 4 of row 3 to 150 -- MASTER INPUT
			set background color of cell 4 of row 3 to {60000, 65535, 60000}
			
			set value of cell 2 of row 4 to "Monthly Burn"
			set value of cell 4 of row 4 to 50000 -- MASTER INPUT
			set value of cell 3 of row 4 to "=D4/D3" -- Dynamic SOL Burn
			set background color of cell 4 of row 4 to {60000, 65535, 60000}
			
			set value of cell 2 of row 5 to "Rent Cost / Byte"
			set value of cell 3 of row 5 to 0.00000927
			set value of cell 4 of row 5 to "=C5*D3"
			
			set value of cell 2 of row 6 to "Tx Fee"
			set value of cell 3 of row 6 to 0.000005
			set value of cell 4 of row 6 to "=C6*D3"
			
			set value of cell 1 of row 8 to "BASELINE VOLUME"
			set font name of cell 1 of row 8 to "Helvetica-Bold"
			set value of cell 2 of row 9 to "Tiny Deploys (386B)"
			set value of cell 3 of row 9 to 500
			set background color of cell 3 of row 9 to {60000, 65535, 60000}
			
			set value of cell 2 of row 10 to "Small Deploys (2KB)"
			set value of cell 3 of row 10 to 200
			set background color of cell 3 of row 10 to {60000, 65535, 60000}
			
			set value of cell 2 of row 11 to "Large Deploys (16KB)"
			set value of cell 3 of row 11 to 20
			set background color of cell 3 of row 11 to {60000, 65535, 60000}
			
			set value of cell 2 of row 12 to "Starting TPS"
			set value of cell 3 of row 12 to 20
			set background color of cell 3 of row 12 to {60000, 65535, 60000}
			
			set value of cell 1 of row 14 to "GROWTH TARGETS"
			set font name of cell 1 of row 14 to "Helvetica-Bold"
			set value of cell 2 of row 15 to "Target TPS"
			set value of cell 3 of row 15 to 100
			set background color of cell 3 of row 15 to {60000, 65535, 60000}
		end tell
	end tell
	
	-- 2. MODEL A (Double Rent)
	tell newDoc
		make new sheet
		set name of last sheet to "ModelA"
		tell sheet "ModelA"
			delete every table
			set tableA to make new table with properties {name:"Calc", row count:16, column count:5}
			tell tableA
				set value of cell 1 of row 1 to "MODEL A: DOUBLE RENT"
				set background color of row 1 to {50000, 50000, 55000}
				set font name of row 1 to "Helvetica-Bold"
				
				set value of cell 2 of row 3 to "Tiny Revenue"
				set value of cell 3 of row 3 to "=(Dashboard::Inputs::C9 * 386 * Dashboard::Inputs::C5) * 2"
				set value of cell 2 of row 4 to "Small Revenue"
				set value of cell 3 of row 4 to "=(Dashboard::Inputs::C10 * 2048 * Dashboard::Inputs::C5) * 2"
				set value of cell 2 of row 5 to "Large Revenue"
				set value of cell 3 of row 5 to "=(Dashboard::Inputs::C11 * 16384 * Dashboard::Inputs::C5) * 2"
				
				set value of cell 2 of row 7 to "Total Deploy Revenue"
				set value of cell 3 of row 7 to "=SUM(C3:C5)"
				set value of cell 2 of row 8 to "Less Solana Rent Cost"
				set value of cell 3 of row 8 to "=(Dashboard::Inputs::C9*386*Dashboard::Inputs::C5) + (Dashboard::Inputs::C10*2048*Dashboard::Inputs::C5) + (Dashboard::Inputs::C11*16384*Dashboard::Inputs::C5)"
				
				set value of cell 2 of row 10 to "Tx Profit"
				set value of cell 3 of row 10 to "=Dashboard::Inputs::C12 * 2592000 * Dashboard::Inputs::C6"
				
				set value of cell 2 of row 12 to "Gross Profit (SOL)"
				set value of cell 3 of row 12 to "=(C7 - C8) + C10"
				
				set value of cell 2 of row 15 to "NET PROFIT (USD)"
				set value of cell 4 of row 15 to "=(C12 - Dashboard::Inputs::C4) * Dashboard::Inputs::D3"
			end tell
		end tell
	end tell
	
	-- 3. MODEL B (Hybrid)
	tell newDoc
		make new sheet
		set name of last sheet to "ModelB"
		tell sheet "ModelB"
			delete every table
			set tableB to make new table with properties {name:"Calc", row count:16, column count:5}
			tell tableB
				set value of cell 1 of row 1 to "MODEL B: HYBRID"
				set background color of row 1 to {50000, 50000, 55000}
				set font name of row 1 to "Helvetica-Bold"
				set value of cell 2 of row 2 to "Base Fee (SOL)"
				set value of cell 3 of row 2 to 0.05
				set background color of cell 3 of row 2 to {60000, 65535, 60000}
				
				set value of cell 2 of row 4 to "Tiny Revenue"
				set value of cell 3 of row 4 to "=Dashboard::Inputs::C9 * ((386 * Dashboard::Inputs::C5) + C2)"
				set value of cell 2 of row 5 to "Small Revenue"
				set value of cell 3 of row 5 to "=Dashboard::Inputs::C10 * (C2 + ((2048 * Dashboard::Inputs::C5) * 2))"
				set value of cell 2 of row 6 to "Large Revenue"
				set value of cell 3 of row 6 to "=Dashboard::Inputs::C11 * (C2 + ((16384 * Dashboard::Inputs::C5) * 2))"
				
				set value of cell 2 of row 8 to "Total Deploy Revenue"
				set value of cell 3 of row 8 to "=SUM(C4:C6)"
				set value of cell 2 of row 9 to "Less Solana Rent Cost"
				set value of cell 3 of row 9 to "=(Dashboard::Inputs::C9*386*Dashboard::Inputs::C5) + (Dashboard::Inputs::C10*2048*Dashboard::Inputs::C5) + (Dashboard::Inputs::C11*16384*Dashboard::Inputs::C5)"
				
				set value of cell 2 of row 11 to "Tx Profit"
				set value of cell 3 of row 11 to "=Dashboard::Inputs::C12 * 2592000 * Dashboard::Inputs::C6"
				
				set value of cell 2 of row 12 to "Gross Profit"
				set value of cell 3 of row 12 to "=(C8 - C9) + C11"
				
				set value of cell 2 of row 15 to "NET PROFIT (USD)"
				set value of cell 4 of row 15 to "=(C12 - Dashboard::Inputs::C4) * Dashboard::Inputs::D3"
			end tell
		end tell
	end tell
	
	-- 4. MODEL C (Growth)
	tell newDoc
		make new sheet
		set name of last sheet to "ModelC"
		tell sheet "ModelC"
			delete every table
			set tableC to make new table with properties {name:"Calc", row count:16, column count:5}
			tell tableC
				set value of cell 1 of row 1 to "MODEL C: GROWTH"
				set background color of row 1 to {50000, 50000, 55000}
				set font name of row 1 to "Helvetica-Bold"
				
				set value of cell 2 of row 2 to "Tiny Flat Fee"
				set value of cell 3 of row 2 to 0.035
				set value of cell 2 of row 3 to "Large Flat Fee"
				set value of cell 3 of row 3 to 0.20
				
				set value of cell 2 of row 5 to "Tiny Revenue"
				set value of cell 3 of row 5 to "=Dashboard::Inputs::C9 * C2"
				set value of cell 2 of row 6 to "Small Revenue"
				set value of cell 3 of row 5 to "=Dashboard::Inputs::C10 * C3"
				set value of cell 2 of row 7 to "Large Revenue"
				set value of cell 3 of row 7 to "=Dashboard::Inputs::C11 * C3"
				
				set value of cell 2 of row 8 to "Total Deploy Revenue"
				set value of cell 3 of row 8 to "=SUM(C5:C7)"
				set value of cell 2 of row 9 to "Less Solana Rent Cost"
				set value of cell 3 of row 9 to "=(Dashboard::Inputs::C9*386*Dashboard::Inputs::C5) + (Dashboard::Inputs::C10*2048*Dashboard::Inputs::C5) + (Dashboard::Inputs::C11*16384*Dashboard::Inputs::C5)"
				
				set value of cell 2 of row 11 to "Tx Profit"
				set value of cell 3 of row 11 to "=Dashboard::Inputs::C12 * 2592000 * Dashboard::Inputs::C6"
				
				set value of cell 2 of row 12 to "Gross Profit"
				set value of cell 3 of row 12 to "=(C8 - C9) + C11"
				
				set value of cell 2 of row 15 to "NET PROFIT (USD)"
				set value of cell 4 of row 15 to "=(C12 - Dashboard::Inputs::C4) * Dashboard::Inputs::D3"
			end tell
		end tell
	end tell
	
	-- 5. VELOCITY & TIMELINE
	tell newDoc
		make new sheet
		set name of last sheet to "Velocity"
		tell sheet "Velocity"
			delete every table
			set velTable to make new table with properties {name:"Timeline", row count:20, column count:5}
			tell velTable
				set background color of range "A1:E1" to {0, 30000, 60000}
				set font name of range "A1:E1" to "Helvetica-Bold"
				set text color of range "A1:E1" to {65535, 65535, 65535}
				set value of cell 1 of row 1 to "FORECAST"
				set value of cell 2 of row 1 to "Metric"
				set value of cell 3 of row 1 to "Model A"
				set value of cell 4 of row 1 to "Model B"
				set value of cell 5 of row 1 to "Model C"
				
				set value of cell 1 of row 3 to "Tiny Cost (USD)"
				set value of cell 3 of row 3 to "= (386 * Dashboard::Inputs::C5 * 2) * Dashboard::Inputs::D3"
				set value of cell 4 of row 3 to "= ((386 * Dashboard::Inputs::C5) + ModelB::Calc::C2) * Dashboard::Inputs::D3"
				set value of cell 5 of row 3 to "= ModelC::Calc::C2 * Dashboard::Inputs::D3"
				
				set value of cell 1 of row 4 to "Velocity Score"
				set value of cell 3 of row 4 to "=IF(C3 < 1, 3.0, IF(C3 < 5, 1.5, 1.0))"
				set value of cell 4 of row 4 to "=IF(D3 < 1, 3.0, IF(D3 < 5, 1.5, 1.0))"
				set value of cell 5 of row 4 to "=IF(E3 < 1, 3.0, IF(E3 < 5, 1.5, 1.0))"
				
				set value of cell 1 of row 6 to "Months to Target"
				set font name of cell 1 of row 6 to "Helvetica-Bold"
				set value of cell 3 of row 6 to "=(Dashboard::Inputs::C15 - Dashboard::Inputs::C12) / (2 * C4)"
				set value of cell 4 of row 6 to "=(Dashboard::Inputs::C15 - Dashboard::Inputs::C12) / (2 * D4)"
				set value of cell 5 of row 6 to "=(Dashboard::Inputs::C15 - Dashboard::Inputs::C12) / (2 * E4)"
			end tell
		end tell
	end tell
	
	-- 6. DASHBOARD SUMMARY (Final dynamic table)
	tell sheet 1 of newDoc
		set summaryTable to make new table with properties {name:"Summary", row count:6, column count:5}
		tell summaryTable
			set value of cell 1 of row 1 to "FINAL COMPARISON"
			set background color of range "A1:E1" to {55000, 55000, 60000}
			set font name of range "A1:E1" to "Helvetica-Bold"
			
			set value of cell 3 of row 1 to "USD Profit"
			set value of cell 4 of row 1 to "Months to Goal"
			
			set value of cell 2 of row 2 to "Model A (Double Rent)"
			set value of cell 3 of row 2 to "=ModelA::Calc::D15"
			set value of cell 4 of row 2 to "=Velocity::Timeline::C6"
			
			set value of cell 2 of row 3 to "Model B (Hybrid)"
			set value of cell 3 of row 3 to "=ModelB::Calc::D15"
			set value of cell 4 of row 3 to "=Velocity::Timeline::D6"
			
			set value of cell 2 of row 4 to "Model C (Growth)"
			set value of cell 3 of row 4 to "=ModelC::Calc::D15"
			set value of cell 4 of row 4 to "=Velocity::Timeline::E6"
		end tell
	end tell
	
	set active sheet of newDoc to sheet 1 of newDoc
end tell
