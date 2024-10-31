#!/usr/bin/env perl
=pod

=head1 NAME

examples/spinner.pl - standard spinner widget

=head1 FEATURES

Demonstrates use of various spinner styles provided by the C<Prima::Spinner> API.

=cut

use strict;
use warnings;
use Tk;
use Tk::ProgressBar;

my $updatePercentage = 0;

my $mw = MainWindow->new;
$mw->Label(-text => 'Focus on your breathing...')->pack;

$mw->ProgressBar(
    -from   => 0,
    -to     => 100,
    -blocks => 100,
    -colors => [0, 'green', 33, 'yellow' , 66, 'red'],
    -variable => \$updatePercentage,
)->pack;

$mw->Button(
    -text    => 'Quit',
    -command => sub { exit },
)->pack;

$mw->after(1000, (\&breath(5)));

MainLoop;

sub breath {
    my $step = shift;
    my $direction = 1;
    while ($direction){
        for (1..20) {
            $updatePercentage += $step;
            $mw->update;
            select(undef, undef, undef, 0.1);
        }
        $direction = 0;
    }
    &breath($step * -1);
    Tk::exit;
}
