import subprocess
import numpy as np
import os
import json
import matplotlib.pyplot as plt
import sys
import glob
from matplotlib.patches import Patch
from matplotlib.ticker import FormatStrFormatter


num_events = 5_000_000
num_warmups = 5
num_iterations = 10

flink = ['flink', 'run', 'queries/flink/target/flink-nexmark-1.0-SNAPSHOT.jar']
rust = ['./queries/rust/target/release/rust-nexmark']


def json_path(n):
    return f'output/experiment{n}-results.json'


main_queries = ["Q4", "Size 100000"]

for (main_systems, include, exclude) in [
    # (['Flink', 'Rust'], ['Flink', 'Rust'], ['FlinkOpt', 'RustOpt']),
    # # (['Flink', 'FlinkOpt'], ['Flink', 'FlinkOpt'], ['Rust', 'RustOpt']),
    # (['Flink', 'FlinkOpt'], ['Flink', 'Rust', 'FlinkOpt'], ['RustOpt']),
    # (['FlinkOpt', 'RustOpt'], ['Flink', 'Rust', 'FlinkOpt', 'RustOpt'], []),
    # (['Flink', 'RustOpt'], ['Flink', 'Rust', 'FlinkOpt', 'RustOpt'], []),
    # (['Rust', 'RustOpt'], ['Flink', 'Rust', 'FlinkOpt', 'RustOpt'], [])
    (['Flink', 'Rust', 'FlinkOpt', 'RustOpt'], [
     'Flink', 'Rust', 'FlinkOpt', 'RustOpt'], [])
]:

    def pdf_path(n, io=False):
        if io:
            return f'output/experiment{n}-{"-".join(main_systems)}-io-plot.pdf'
        else:
            return f'output/experiment{n}-{"-".join(main_systems)}-plot.pdf'

    def setup():
        if not os.path.exists('data'):
            os.mkdir('data')

        if os.path.exists('output'):
            for f in glob.glob('output/*'):
                os.remove(f)
        else:
            os.mkdir('output')

        # Start Flink cluster if not running
        p = subprocess.run(['jps'], capture_output=True)
        if 'TaskManager' not in p.stdout.decode('utf-8'):
            print('Starting Flink Cluster')
            subprocess.run(['start-cluster.sh'], check=True)
            # Ensure that the taskmanager has the correct configuration
            print('Configuring TaskManager')
            subprocess.run(['taskmanager.sh', 'stop'], check=True)
            subprocess.run(['taskmanager.sh', 'start'], check=True)
        else:
            print('Flink Cluster already online')

        # Build Rust project
        subprocess.run(['cargo', 'build', '--release',
                        '--manifest-path=queries/rust/Cargo.toml'])

        # Build Java project
        subprocess.run(['mvn', 'clean', 'package',
                       '-f', 'queries/flink/pom.xml'])

        # Build Data Generator
        subprocess.run(['cargo', 'build', '--release',
                        '--manifest-path=data-generator/Cargo.toml'])

    def generate(s, n=num_events):
        b = 'b' in s
        a = 'a' in s
        p = 'p' in s
        path = f'data/data-n{n}-{s}/'
        if not os.path.exists(path):
            cmd = ['./data-generator/target/release/data-generator',
                   '--num-events=' + str(n),
                   '--dir=' + path]
            if b:
                cmd.append('--bids')
            if a:
                cmd.append('--auctions')
            if p:
                cmd.append('--persons')
            print(f'Generating data: {" ".join(cmd)} ')
            subprocess.run(cmd, check=True)
        return path

    # Run experiments

    io_cache = {}

    def measure_query(program, query, data, extra_args=[]):
        y, yerr = measure(program, query, data, extra_args)
        y_io, yerr_io = measure_io(program, data)
        print(f'Execution time: {y} ± {yerr} (I/O: {y_io} ± {yerr_io})')
        return y, yerr, y_io, yerr_io

    def measure_io(program, data):
        if (tuple(program), data) in io_cache:
            print(f'IO Cache hit {" ".join(program)} {data}')
            return io_cache[(tuple(program), data)]
        else:
            y, yerr = measure(program, 'io', data)
            io_cache[(tuple(program), data)] = (y, yerr)
            return y, yerr

    def measure(program, query, data, extra_args=[]):
        execution_times = []
        cmd = program + [data, query] + extra_args
        print(f'Measuring: {" ".join(cmd)}')
        for iteration in range(num_iterations + num_warmups):
            if iteration == num_warmups and num_warmups > 0:
                print('Warmup done, starting measurements')
            output = subprocess.run(
                cmd,
                capture_output=True,
                text=True).stderr
            seconds = int(output) / 1000
            if iteration >= num_warmups:
                execution_times.append(seconds)
            print(f'Iteration {iteration+1}/{num_iterations + num_warmups} '
                  f'execution time: {seconds} sec')
        y = np.mean(execution_times)
        yerr = np.std(execution_times)
        return y, yerr

    def read(path):
        with open(path, 'r') as file:
            return json.load(file)

    def write(path, obj):
        with open(path, 'w') as file:
            file.write(json.dumps(obj, indent=4))

    def run_experiment1():
        def f(program, query, data):
            return measure_query(program, query, data)
        obj = {
            'Q1': {
                'Flink': f(flink, 'q1', generate('b')),
                'Rust': f(rust, 'q1', generate('b')),
                'FlinkOpt': f(flink, 'q1-opt', generate('b')),
                'RustOpt': f(rust, 'q1-opt', generate('b'))
            },
            'Q2': {
                'Flink': f(flink, 'q2', generate('b')),
                'Rust': f(rust, 'q2', generate('b')),
                'FlinkOpt': f(flink, 'q2-opt', generate('b')),
                'RustOpt': f(rust, 'q2-opt', generate('b'))
            },
            'Q3': {
                'Flink': f(flink, 'q3', generate('ap')),
                'Rust': f(rust, 'q3', generate('ap')),
                'FlinkOpt': f(flink, 'q3-opt', generate('ap')),
                'RustOpt': f(rust, 'q3-opt', generate('ap'))
            },
            'Q4': {
                'Flink': f(flink, 'q4', generate('ab')),
                'Rust': f(rust, 'q4', generate('ab')),
                'FlinkOpt': f(flink, 'q4-opt', generate('ab')),
                'RustOpt': f(rust, 'q4-opt', generate('ab'))
            },
            'Q5': {
                'Flink': f(flink, 'q5', generate('b')),
                'Rust': f(rust, 'q5', generate('b')),
                'FlinkOpt': f(flink, 'q5-opt', generate('b')),
                'RustOpt': f(rust, 'q5-opt', generate('b'))
            },
            'Q6': {
                'Flink': f(flink, 'q6', generate('ab')),
                'Rust': f(rust, 'q6', generate('ab')),
                'FlinkOpt': f(flink, 'q6-opt', generate('ab')),
                'RustOpt': f(rust, 'q6-opt', generate('ab'))
            },
            'Q7': {
                'Flink': f(flink, 'q7', generate('b')),
                'Rust': f(rust, 'q7', generate('b')),
                'FlinkOpt': f(flink, 'q7-opt', generate('b')),
                'RustOpt': f(rust, 'q7-opt', generate('b'))
            },
            'Q8': {
                'Flink': f(flink, 'q8', generate('ap')),
                'Rust': f(rust, 'q8', generate('ap')),
                'FlinkOpt': f(flink, 'q8-opt', generate('ap')),
                'RustOpt': f(rust, 'q8-opt', generate('ap'))
            }
        }
        write(json_path(1), obj)

    def run_experiment2():
        slide = '100'
        data = generate('b')

        def f(program, size, query):
            return measure_query(program, query, data, extra_args=[size, slide])

        def g(program, size, query):
            y_io, yerr_io = measure_io(program, data)
            return 'Timeout', 'N/A', y_io, yerr_io

        obj = {
            'Size 100': {
                'Flink': f(flink, '100', 'qw'),
                'Rust': f(rust, '100', 'qw'),
                'FlinkOpt': f(flink, '100', 'qw-opt'),
                'RustOpt': f(rust, '100', 'qw-opt'),
            },
            'Size 1000': {
                'Flink': f(flink, '1000', 'qw'),
                'Rust': f(rust, '1000', 'qw'),
                'FlinkOpt': f(flink, '1000', 'qw-opt'),
                'RustOpt': f(rust, '1000', 'qw-opt'),
            },
            'Size 10000': {
                'Flink': f(flink, '10000', 'qw'),
                'Rust': f(rust, '10000', 'qw'),
                'FlinkOpt': f(flink, '10000', 'qw-opt'),
                'RustOpt': f(rust, '10000', 'qw-opt'),
            },
            'Size 100000': {
                'Rust': f(rust, '100000', 'qw'),
                'Flink': g(flink, '100000', 'qw'),
                'RustOpt': f(rust, '100000', 'qw-opt'),
                'FlinkOpt': g(flink, '100000', 'qw-opt'),
            }
        }
        write(json_path(2), obj)

    def plot(result_json_path, plot_path, tex=False, io=False):
        obj = read(result_json_path)
        ymax = max([y for query in obj.values()
                    for y, _, _, _ in query.values() if y != 'Timeout'])
        ylim = ymax * 1.1

        SMALL_SIZE = 8
        MEDIUM_SIZE = 10
        BIGGER_SIZE = 10

        plt.rc('font', size=SMALL_SIZE)
        plt.rc('axes', titlesize=SMALL_SIZE)
        plt.rc('axes', labelsize=MEDIUM_SIZE)
        plt.rc('xtick', labelsize=SMALL_SIZE)
        plt.rc('ytick', labelsize=SMALL_SIZE)
        plt.rc('legend', fontsize=SMALL_SIZE)
        plt.rc('figure', titlesize=BIGGER_SIZE)
        if tex:
            plt.rcParams['text.usetex'] = True
        plt.rcParams['pdf.fonttype'] = 42
        plt.rcParams['ps.fonttype'] = 42
        plt.rcParams["font.family"] = "serif"
        plt.rcParams["font.size"] = "8"

        fig, ax = plt.subplots(figsize=(8/3, 1.8))

        ax.yaxis.set_major_formatter(FormatStrFormatter('%.0f'))
        ax.set_ylim([0, ylim])

        bar_width = 0.2

        system_names = ['Flink', 'Rust', 'FlinkOpt', 'RustOpt']

        colors = plt.cm.Accent(np.linspace(0, 0.5, len(system_names)))
        color_map = dict(zip(system_names, colors))
        hatch_map = dict(zip(system_names, ['/', 'x', '.', 'o']))

        for query_index, (query, systems) in enumerate(obj.items()):
            offset = (len(systems) - 1) * bar_width / 2
            for system_index, (system, metrics) in enumerate(systems.items()):
                x = query_index - offset + system_index * bar_width
                y, y_err, y_io, y_io_err = metrics
                hatch = hatch_map[system]
                color = color_map[system]
                show = query in main_queries and system in main_systems
                if system in main_systems:
                    alpha = 1
                elif system in include:
                    alpha = 0.2
                elif system in exclude:
                    alpha = 0
                if y == 'Timeout':
                    # Write vertical timeout in the bar
                    ax.text(x, y_io+1, r'Timeout ($>$5min)' if show else '',
                            color='black',
                            rotation=90, ha='center', va='bottom')
                    ax.bar(x, ylim+1, bar_width,
                           color=color,
                           edgecolor='black',
                           alpha=alpha,
                           linewidth=0.5)
                    if io:
                        ax.bar(x, y_io, bar_width, yerr=y_io_err,
                               color='lightgray',
                               edgecolor='black',
                               linewidth=0.5,
                               alpha=0.8)
                else:
                    if not io:
                        y = y - y_io

                    ax.bar(x, y, bar_width,
                           yerr=y_err if system not in exclude else 0,
                           color=color,
                           edgecolor='black',
                           alpha=alpha,
                           linewidth=0.5,
                           hatch=hatch)
                    if io:
                        ax.bar(x, y_io, bar_width,
                               yerr=y_io_err if system not in exclude else 0,
                               color='lightgray',
                               edgecolor='black',
                               linewidth=0.5,
                               alpha=alpha)

        ax.set_xticks(np.arange(len(obj)))
        ax.set_xticklabels(obj.keys())

        ax.set_ylabel('Execution Time (s)')

        handles = [Patch(
            label=system if system in include else '',
            facecolor=color_map[system],
            alpha=1 if system in include else 0,
            hatch=hatch_map[system],
            edgecolor='black' if system in include else 'none',
            linewidth=0.5
        ) for system in system_names]

        if io:
            handles.append(Patch(label='I/O',
                                 facecolor='lightgray',
                                 linewidth=0.5,
                                 edgecolor='black'))

        ax.legend(handles=handles,
                  title='System',
                  fontsize="7",
                  handlelength=0.75,
                  handletextpad=0.5,
                  loc="upper right")

        plt.tight_layout()
        plt.savefig(plot_path, bbox_inches='tight', dpi=1200, transparent=True)
        print(f'Plot saved to {plot_path}')

    def plot_experiment1(tex=False):
        plot(json_path(1), pdf_path(1, io=True), tex=tex, io=True)
        plot(json_path(1), pdf_path(1, io=False), tex=tex, io=False)

    def plot_experiment2(twocolumn=False, tex=False):
        plot(json_path(2), pdf_path(2, io=True), tex=tex, io=True)
        plot(json_path(2), pdf_path(2, io=False), tex=tex, io=False)

    if len(sys.argv) > 1 and sys.argv[1] == 'plot':
        plot_experiment1(tex=True)
        plot_experiment2(tex=True)
    else:
        setup()

        run_experiment1()
        run_experiment2()

        plot_experiment1(tex=False)
        plot_experiment2(tex=False)
