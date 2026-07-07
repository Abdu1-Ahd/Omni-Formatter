#!/usr/bin/perl
use strict;
use warnings;
use utf8;
use JSON;
use List::Util qw(sum max min first);

# ── CASE 1: Variable declarations — spacing ────────────────────────────────
my $name    = "Alice";
my $age     =  30;
my @hobbies = ('reading','coding','hiking');
my %config  = (host => 'localhost', port => 5432, db => 'mydb');

# ── CASE 2: Subroutines ────────────────────────────────────────────────────
sub greet {
    my ($name,$greeting) = @_;
    $greeting //= "Hello";
    return "$greeting, $name!";
}

sub classify {
    my $n = shift;
    return "negative" if $n < 0;
    return "zero"     if $n == 0;
    return "small"    if $n < 10;
    return "large";
}

# ── CASE 3: References and data structures ─────────────────────────────────
my $user = {
    id    => 1,
    name  => "Alice",
    email => 'alice@example.com',
    roles => ['admin', 'user'],
};

my @users = (
    { id => 1, name => "Alice" , email => "alice\@example.com" },
    { id => 2, name => "Bob"   , email => "bob\@example.com"   },
);

# ── CASE 4: Regular expressions ────────────────────────────────────────────
sub is_valid_email {
    my $email = shift;
    return $email =~ /^[a-zA-Z0-9._%+-]+\@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
}

sub extract_numbers {
    my $text = shift;
    my @nums = ($text =~ /(\d+)/g);
    return @nums;
}

# ── CASE 5: File I/O and map/grep ─────────────────────────────────────────
sub process_lines {
    my $filename = shift;
    open(my $fh, '<', $filename) or die "Cannot open: $!";
    my @lines = grep { /\S/ } map { chomp; $_ } <$fh>;
    close $fh;
    return @lines;
}

# ── CASE 6: OOP with bless ────────────────────────────────────────────────
package User;

sub new {
    my ($class, %args) = @_;
    return bless {
        id    => $args{id},
        name  => $args{name},
        email => $args{email},
    }, $class;
}

sub greet { "Hello, " . $_[0]->{name} . "!" }

package main;

# ── CASE 7: Trailing whitespace ────────────────────────────────────────────
my $result = greet("World");   
print "$result\n";  
