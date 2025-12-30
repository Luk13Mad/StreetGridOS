import json
import argparse
import pandas as pd
import matplotlib.pyplot as plt
from model import Microgrid

def main():
    parser = argparse.ArgumentParser(description="StreetGridOS Digital Twin Simulation")
    parser.add_argument("scenario", help="Path to scenario JSON file")
    parser.add_argument("--plot", action="store_true", help="Generate a plot of the simulation")
    args = parser.parse_args()

    with open(args.scenario, 'r') as f:
        scenario_data = json.load(f)

    print(f"Running scenario: {scenario_data['scenario_name']}")

    microgrid = Microgrid(scenario_data['nodes'])
    duration = scenario_data['duration_hours']

    results = []

    # Simulate hour by hour
    for hour in range(duration):
        step_result = microgrid.run_step(hour)
        results.append(step_result)

    df = pd.DataFrame(results)
    print("\nSimulation Results Summary:")
    print(df[['hour', 'soc', 'total_solar_kw', 'total_load_kw', 'net_grid_kwh', 'status']].to_string(index=False))

    # Check if grid crashed (any large deficit)
    failures = df[df['net_grid_kwh'] < -0.1]
    if not failures.empty:
        print("\nFAILURE: Grid collapsed during the following hours:")
        print(failures)
    else:
        print("\nSUCCESS: Grid remained stable for 24 hours.")

    if args.plot:
        plt.figure(figsize=(10, 6))
        plt.plot(df['hour'], df['total_solar_kw'], label='Total Solar (kW)')
        plt.plot(df['hour'], df['total_load_kw'], label='Total Load (kW)')
        plt.plot(df['hour'], df['net_grid_kwh'], label='Net Grid Energy (kWh)', linestyle='--')

        # Plot Battery SOC on secondary axis
        ax2 = plt.gca().twinx()
        ax2.plot(df['hour'], df['soc'], label='Battery SOC', color='green', linestyle=':')
        ax2.set_ylabel('State of Charge (0-1)')
        ax2.set_ylim(0, 1.1)

        plt.title(f"Simulation: {scenario_data['scenario_name']}")
        plt.xlabel('Hour of Day')
        plt.ylabel('Power/Energy')
        plt.legend(loc='upper left')
        plt.grid(True)

        output_file = "simulation_result.png"
        plt.savefig(output_file)
        print(f"\nPlot saved to {output_file}")

if __name__ == "__main__":
    main()
