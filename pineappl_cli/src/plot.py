#!/usr/bin/env python3

import math
import matplotlib.pyplot as plt
import numpy as np
import pickle

def percent_diff(a, b):
    return (a / b - 1.0) * 100.0

def ylimits(axis):
    # extract the y limits *not* considering margins
    margins = axis.margins()
    axis.margins(y=0.0)
    min, max = axis.get_ylim()
    axis.margins(y=margins[1])

    inc = 1.0

    if (max - min) > 100.0:
        min = -50.0
        max = 50.0
        inc = 25.0
    elif (max - min) > 30.5:
        inc = 10.0
    elif (max - min) > 20.5:
        inc = 5.0
    elif (max - min) > 10.5:
        inc = 2.0
    elif (max - min) < 3.0:
        inc = 0.5

    min = math.floor(min / inc) * inc
    max = math.ceil(max / inc) * inc

    return [min, max, inc]

def plot_int(axis, **kwargs):
    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()
    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')

    xmin = np.array([])
    xmax = np.array([])
    x = np.array([])
    y = np.array([])

    for index, i in enumerate(kwargs['pdf_results']):
        label, ycentral, ymin, ymax = i
        x = np.append(x, ycentral[:-1])
        xmin = np.append(xmin, ymin[:-1])
        xmax = np.append(xmax, ymax[:-1])
        y = np.append(y, label)

        # draw one- and two-sigma bands
        if label == 'CENTRAL-PDF':
            axis.axvspan(xmin[-1], xmax[-1], alpha=0.3, color='royalblue', linewidth=0)
            # TODO: this is only correct for MC PDF uncertainties
            axis.axvspan(x[-1] - 2.0 * (x[-1] - xmin[-1]), x[-1] + 2.0 * (xmax[-1] - x[-1]), alpha=0.1, color='royalblue', linewidth=0)

    axis.errorbar(x, y, xerr=(x - xmin, xmax - x), fmt='.', capsize=3, markersize=5, linewidth=1.5)
    axis.margins(x=0.1, y=0.1)

def plot_abs(axis, **kwargs):
    x = kwargs['x']
    y = kwargs['y']
    ymin = kwargs['ymin']
    ymax = kwargs['ymax']
    ylog = kwargs['ylog']
    ylabel = kwargs['ylabel']
    slice_label = kwargs['slice_label']

    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()
    axis.set_yscale('log' if ylog else 'linear')
    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')
    axis.step(x, y, 'royalblue', linewidth=1.0, where='post', label=slice_label)
    axis.fill_between(x, ymin, ymax, alpha=0.4, color='royalblue', linewidth=0.5, step='post')
    axis.set_ylabel(ylabel)

    if slice_label != '':
        axis.legend(fontsize='xx-small', frameon=False)

def plot_abs_pdfs(axis, **kwargs):
    x = kwargs['x']
    ylog = kwargs['ylog']
    ylabel = kwargs['ylabel']
    slice_label = kwargs['slice_label']
    pdf_uncertainties = kwargs['pdf_results']
    channels = kwargs['channels']

    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()
    axis.set_yscale('log' if ylog else 'linear')
    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')
    axis.set_ylabel(ylabel)

    colors = plt.rcParams['axes.prop_cycle'].by_key()['color']
    for index, i in enumerate(pdf_uncertainties):
        label, y, ymin, ymax = i
        axis.step(x, y, color=colors[index], linewidth=1.0, where='post')
        axis.fill_between(x, ymin, ymax, alpha=0.4, color=colors[index], label=label, linewidth=0.5, step='post')

    linestyles = ['--', ':']
    for index, i in enumerate(channels):
        if index >= len(linestyles):
            break

        label, y = i
        axis.step(x, y, color=colors[0], label=label, linestyle=linestyles[index], linewidth=1.0, where='post')

    axis.legend(bbox_to_anchor=(0,-0.24,1,0.2), loc='upper left', mode='expand', borderaxespad=0, ncol=min(4, len(pdf_uncertainties) + 2), fontsize='x-small', frameon=False, borderpad=0)

    if slice_label != '':
        t = axis.text(0.98, 0.98, slice_label, horizontalalignment='right', verticalalignment='top', transform=axis.transAxes, fontsize='x-small')
        t.set_bbox({{ 'alpha': 0.7, 'boxstyle': 'square, pad=0.0', 'edgecolor': 'white', 'facecolor': 'white' }})

def plot_rel_ewonoff(axis, **kwargs):
    x = kwargs['x']
    y = percent_diff(kwargs['y'], kwargs['qcd_y'])
    qcd_y = percent_diff(kwargs['qcd_y'], kwargs['qcd_y'])
    qcd_ymin = percent_diff(kwargs['qcd_min'], kwargs['qcd_y'])
    qcd_ymax = percent_diff(kwargs['qcd_max'], kwargs['qcd_y'])
    ymin = percent_diff(kwargs['ymin'], kwargs['qcd_y'])
    ymax = percent_diff(kwargs['ymax'], kwargs['qcd_y'])
    pdf_min = abs(percent_diff(kwargs['pdf_results'][0][2], kwargs['pdf_results'][0][1]))[:-1]
    pdf_max = abs(percent_diff(kwargs['pdf_results'][0][3], kwargs['pdf_results'][0][1]))[:-1]
    mid = kwargs['mid']

    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()
    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')
    axis.step(x, qcd_y, 'red', label='NLO QCD', linewidth=1.0, where='post')
    #axis.fill_between(x, qcd_ymin, qcd_ymax, alpha=0.4, color='red', label='7-p.\ scale var.', linewidth=0.5, step='post')
    axis.step(x, y, 'royalblue', label='NLO QCD+EW', linewidth=1.0, where='post')
    axis.fill_between(x, ymin, ymax, alpha=0.4, color='royalblue', label='7-p.\ scale var.', linewidth=0.5, step='post')
    axis.errorbar(mid, y[:-1], yerr=(pdf_min, pdf_max), color='royalblue', label='PDF uncertainty', fmt='.', capsize=1, markersize=0, linewidth=1)
    axis.set_ylabel('NLO EW on/off [\si{{\percent}}]')
    axis.legend(bbox_to_anchor=(0,1.03,1,0.2), loc='lower left', mode='expand', borderaxespad=0, ncol=4, fontsize='x-small', frameon=False, borderpad=0)

def plot_rel_pdfunc(axis, **kwargs):
    x = kwargs['x']
    pdf_uncertainties = kwargs['pdf_results']
    colors = plt.rcParams['axes.prop_cycle'].by_key()['color']

    #ymins = np.asmatrix([(ymin / y - 1.0) * 100 for label, y, ymin, ymax in pdf_uncertainties])
    #ymaxs = np.asmatrix([(ymax / y - 1.0) * 100 for label, y, ymin, ymax in pdf_uncertainties])

    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')
    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()

    for index, i in enumerate(pdf_uncertainties):
        label, y, ymin, ymax = i
        ymin = percent_diff(ymin, y)
        ymax = percent_diff(ymax, y)
        axis.step(x, ymax, color=colors[index], label=label, linewidth=1, where='post')
        axis.step(x, ymin, color=colors[index], linewidth=1, where='post')

    #axis.legend(fontsize='xx-small') #rel_pdfunc
    axis.set_ylabel('PDF uncertainty [\si{{\percent}}]')

    this_ylim = ylimits(axis)

    if False:#SAVE-YLIM-PDFUNC
        with open('ylim-pdfunc', 'wb') as f:
            pickle.dump(this_ylim, f)

    if False:#LOAD-YLIM-PDFUNC
        resave = False

        with open('ylim-pdfunc', 'rb') as f:
            ylim = pickle.load(f)

        if ylim[0] < this_ylim[0]:
            this_ylim[0] = ylim[0]
            resave = True

        if ylim[1] > this_ylim[1]:
            this_ylim[1] = ylim[1]
            resave = True

        if ylim[2] > this_ylim[2]:
            this_ylim[2] = ylim[2]
            resave = True

        if resave:
            with open('ylim-pdfunc', 'wb') as f:
                pickle.dump(this_ylim, f)

    axis.set_yticks(np.arange(this_ylim[0], this_ylim[1] + this_ylim[2], this_ylim[2]))
    space = 0.05 * (this_ylim[1] - this_ylim[0])
    axis.set_ylim((this_ylim[0] - space, this_ylim[1] + space))

def plot_rel_pdfpull(axis, **kwargs):
    central_y = kwargs['pdf_results'][0][1]
    central_ymin = kwargs['pdf_results'][0][2]
    central_ymax = kwargs['pdf_results'][0][3]
    pdf_uncertainties = kwargs['pdf_results']
    colors = plt.rcParams['axes.prop_cycle'].by_key()['color']
    x = kwargs['x']
    y = kwargs['y']

    axis.tick_params(axis='both', left=True, right=True, top=True, bottom=True, which='both', direction='in', width=0.5, zorder=10.0)
    axis.minorticks_on()
    axis.set_axisbelow(True)
    axis.grid(linestyle='dotted')

    for index, i in enumerate(pdf_uncertainties):
        label, y, ymin, ymax = i
        diff = y - central_y
        yerr = np.where(diff > 0.0, y - ymin, ymax - y)
        #pull_avg = (y - central_y) / np.sqrt(np.power(0.5 * (ymax - ymin), 2) + np.power(0.5 * (central_ymax - central_ymin), 2))
        pull = (y - central_y) / np.sqrt(np.power(yerr, 2) + np.power(0.5 * (central_ymax - central_ymin), 2))

        #axis.fill_between(x, pull, pull_avg, alpha=0.4, color=colors[index], label='sym.\ pull', linewidth=0.5, step='post', zorder=2 * index)
        axis.step(x, pull, color=colors[index], label=label, linewidth=1, where='post', zorder=2 * index + 1)

    axis.legend(bbox_to_anchor=(0,1.03,1,0.2), loc='lower left', mode='expand', borderaxespad=0, ncol=len(pdf_uncertainties), fontsize='x-small', frameon=False, borderpad=0) #rel_pdfpull
    axis.set_ylabel('Pull [$\sigma$]')
    #axis.set_title('Comparison with ' + pdf_uncertainties[0][0], fontdict={{'fontsize': 9}}, loc='left')

    this_ylim = ylimits(axis)

    if False:#SAVE-YLIM-PDFPULL
        with open('ylim-pdfpull', 'wb') as f:
            pickle.dump(this_ylim, f)

    if False:#LOAD-YLIM-PDFPULL
        resave = False

        with open('ylim-pdfpull', 'rb') as f:
            ylim = pickle.load(f)

        if ylim[0] < this_ylim[0]:
            this_ylim[0] = ylim[0]
            resave = True

        if ylim[1] > this_ylim[1]:
            this_ylim[1] = ylim[1]
            resave = True

        if ylim[2] > this_ylim[2]:
            this_ylim[2] = ylim[2]
            resave = True

        if resave:
            with open('ylim-pdfpull', 'wb') as f:
                pickle.dump(this_ylim, f)

    axis.set_yticks(np.arange(this_ylim[0], this_ylim[1] + this_ylim[2], this_ylim[2]))
    space = 0.05 * (this_ylim[1] - this_ylim[0])
    axis.set_ylim((this_ylim[0] - space, this_ylim[1] + space))

def main():
    panels = [
        plot_abs,
        #plot_abs_pdfs,
        plot_rel_ewonoff,
    ]

    data_slices = data()

    if len(data_slices[0]['pdf_results']) > 1:
        panels.extend([
            plot_rel_pdfunc,
            plot_rel_pdfpull,
        ])

    plt.rc('text', usetex=True)
    plt.rc('text.latex', preamble=r'\usepackage{{siunitx}}\usepackage{{lmodern}}')
    plt.rc('font', family='serif', size=14.0)
    plt.rc('axes', labelsize='small')
    plt.rc('pdf', compression=0)

    xaxis = '{xaxis}'
    xunit = metadata().get(xaxis + '_unit', '')
    xlabel = metadata()[xaxis + '_label_tex'] + (r' [\si{{' + xunit + r'}}]' if xunit != '' else '')
    ylabel = metadata()['y_label_tex'] + r' [\si{{' + metadata()['y_unit'] + r'}}]'
    ylog = xunit != ''
    description = metadata()['description']

    if len(data_slices[0]['x']) == 2:
        panels = [ plot_int ]
        xlabel = ylabel
        plt.rc('figure', figsize=(4.2,2.6))
    else:
        plt.rc('figure', figsize=(6.4,len(panels)*2.4))

    for index, dict in enumerate(data_slices):
        dict['xlabel'] = xlabel
        dict['ylabel'] = ylabel
        dict['ylog'] = ylog

        figure, axes = plt.subplots(len(panels), 1, constrained_layout=True, sharex=True, squeeze=False)
        figure.set_constrained_layout_pads(hspace=0, wspace=0)

        if len(dict['x']) != 2 and xunit != '':
            axes[0, 0].set_xscale('log')

        axes[ 0, 0].set_title(description)
        axes[-1, 0].set_xlabel(xlabel)

        for plot, axis in zip(panels, axes[:, 0]):
            plot(axis, **dict)

        name = '{output}' if len(data_slices) == 1 else '{output}-{{}}'.format(index)
        figure.savefig(name + '.pdf')
        plt.close(figure)

def data():
    return {data}

def metadata():
    return {{
{metadata}
    }}

if __name__ == '__main__':
    main()
