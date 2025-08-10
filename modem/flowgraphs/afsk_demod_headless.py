#!/usr/bin/env python3
# -*- coding: utf-8 -*-

#
# SPDX-License-Identifier: GPL-3.0
#
# GNU Radio Python Flow Graph
# Title: AFSK demod
# Author: lazcanoluca
# GNU Radio version: 3.10.12.0

from gnuradio import analog
import math
from gnuradio import blocks
from gnuradio import digital
from gnuradio import filter
from gnuradio.filter import firdes
from gnuradio import gr
from gnuradio.fft import window
import sys
import signal
from argparse import ArgumentParser
from gnuradio.eng_arg import eng_float, intx
from gnuradio import eng_notation
from gnuradio import zeromq
import threading




class afsk_demod_headless(gr.top_block):

    def __init__(self):
        gr.top_block.__init__(self, "AFSK demod", catch_exceptions=True)
        self.flowgraph_started = threading.Event()

        ##################################################
        # Variables
        ##################################################
        self.samp_rate = samp_rate = 48000
        self.pi = pi = 3.1415
        self.freq_deviation = freq_deviation = 1000
        self.baud_rate = baud_rate = 1200

        ##################################################
        # Blocks
        ##################################################

        self.zeromq_sub_source_0 = zeromq.sub_source(gr.sizeof_gr_complex, 1, 'tcp://127.0.0.1:5556', 100, False, (-1), '', False)
        self.zeromq_pub_sink_0 = zeromq.pub_sink(gr.sizeof_char, 1, 'tcp://127.0.0.1:5557', 0, False, (-1), '', True, True)
        self.rational_resampler_xxx_0 = filter.rational_resampler_ccc(
                interpolation=1,
                decimation=40,
                taps=[],
                fractional_bw=0.4)
        self.digital_clock_recovery_mm_xx_0 = digital.clock_recovery_mm_ff((samp_rate/baud_rate), (0.25*0.175*0.175), 0.5, 0.175, 0.005)
        self.digital_binary_slicer_fb_0_0 = digital.binary_slicer_fb()
        self.blocks_multiply_xx_1 = blocks.multiply_vcc(1)
        self.blocks_add_const_vxx_0 = blocks.add_const_ff((-0.5))
        self.analog_sig_source_x_0_0 = analog.sig_source_c(samp_rate, analog.GR_SIN_WAVE, (-1200), 1, 0, 0)
        self.analog_quadrature_demod_cf_0 = analog.quadrature_demod_cf((samp_rate/(pi*freq_deviation*2)))


        ##################################################
        # Connections
        ##################################################
        self.connect((self.analog_quadrature_demod_cf_0, 0), (self.digital_clock_recovery_mm_xx_0, 0))
        self.connect((self.analog_sig_source_x_0_0, 0), (self.blocks_multiply_xx_1, 1))
        self.connect((self.blocks_add_const_vxx_0, 0), (self.digital_binary_slicer_fb_0_0, 0))
        self.connect((self.blocks_multiply_xx_1, 0), (self.analog_quadrature_demod_cf_0, 0))
        self.connect((self.digital_binary_slicer_fb_0_0, 0), (self.zeromq_pub_sink_0, 0))
        self.connect((self.digital_clock_recovery_mm_xx_0, 0), (self.blocks_add_const_vxx_0, 0))
        self.connect((self.rational_resampler_xxx_0, 0), (self.blocks_multiply_xx_1, 0))
        self.connect((self.zeromq_sub_source_0, 0), (self.rational_resampler_xxx_0, 0))


    def get_samp_rate(self):
        return self.samp_rate

    def set_samp_rate(self, samp_rate):
        self.samp_rate = samp_rate
        self.analog_quadrature_demod_cf_0.set_gain((self.samp_rate/(self.pi*self.freq_deviation*2)))
        self.analog_sig_source_x_0_0.set_sampling_freq(self.samp_rate)
        self.digital_clock_recovery_mm_xx_0.set_omega((self.samp_rate/self.baud_rate))

    def get_pi(self):
        return self.pi

    def set_pi(self, pi):
        self.pi = pi
        self.analog_quadrature_demod_cf_0.set_gain((self.samp_rate/(self.pi*self.freq_deviation*2)))

    def get_freq_deviation(self):
        return self.freq_deviation

    def set_freq_deviation(self, freq_deviation):
        self.freq_deviation = freq_deviation
        self.analog_quadrature_demod_cf_0.set_gain((self.samp_rate/(self.pi*self.freq_deviation*2)))

    def get_baud_rate(self):
        return self.baud_rate

    def set_baud_rate(self, baud_rate):
        self.baud_rate = baud_rate
        self.digital_clock_recovery_mm_xx_0.set_omega((self.samp_rate/self.baud_rate))




def main(top_block_cls=afsk_demod_headless, options=None):
    tb = top_block_cls()

    def sig_handler(sig=None, frame=None):
        tb.stop()
        tb.wait()

        sys.exit(0)

    signal.signal(signal.SIGINT, sig_handler)
    signal.signal(signal.SIGTERM, sig_handler)

    tb.start()
    tb.flowgraph_started.set()

    try:
        input('Press Enter to quit: ')
    except EOFError:
        pass
    tb.stop()
    tb.wait()


if __name__ == '__main__':
    main()
