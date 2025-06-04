# Home Buyer Calculator

A terminal-based mortgage calculator that helps you understand the true cost of homeownership.

## Features

- **Interactive TUI**: Navigate through inputs using keyboard shortcuts
- **Comprehensive cost analysis**: Includes principal, interest, taxes, insurance, HOA fees, maintenance, and PMI
- **Cost of capital tracking**: Shows opportunity cost of equity tied up in the home
- **Month-by-month breakdown**: View detailed payment schedules
- **Export to CSV**: Save your analysis for further review
- **Flexible inputs**: Enter costs as percentages or fixed amounts

## Installation

```bash
cargo build --release
```

## Usage

```bash
cargo run
```

### Navigation

- **Enter/l/→**: Next field
- **Esc/h/←**: Previous field  
- **Tab**: Toggle between percentage and dollar amount (where applicable)
- **q**: Quit

### Input Fields

1. **House Value**: Purchase price of the home
2. **Down Payment**: Initial payment (% or $)
3. **HOA Fee**: Monthly homeowners association fee
4. **Interest Rate**: Annual mortgage interest rate (%)
5. **Property Tax**: Annual tax (% of home value or fixed $)
6. **Insurance**: Homeowners insurance (% of home value or fixed $)
7. **Maintenance**: Expected repair costs (% of home value or fixed $)
8. **PMI**: Private mortgage insurance if down payment < 20% (% of loan or fixed $)
9. **House Appreciation**: Expected annual home value change (%)
10. **Loan Term**: Mortgage duration in years
11. **Extra Principal**: Optional additional monthly payment

### Spreadsheet View

- **j/k or ↑/↓**: Navigate rows
- **g/G**: Jump to top/bottom
- **Ctrl+d/u**: Page down/up
- **s**: View summary
- **e**: Export to CSV
- **h/←**: Back to inputs

### Key Metrics

- **Actual Payment**: Total monthly payment including all costs
- **Cost of Capital**: Opportunity cost of equity (what you could earn if invested elsewhere)
- **Waste Cost**: All non-principal payments plus cost of capital
- **Equity**: Home value minus remaining loan balance

## Export

The calculator can export two CSV files:
- `mortgage_spreadsheet.csv`: Month-by-month breakdown
- `mortgage_analysis.csv`: Complete analysis with summary statistics

## Build Requirements

- Rust 1.70+
- Terminal with Unicode support
