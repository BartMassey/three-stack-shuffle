#include <array>
#include <bit>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <map>
#include <stdexcept>
#include <string>
#include <vector>

using Permutation = std::array<uint8_t, 8>;

void rotate_segment(Permutation& p, int first, int middle, int last) {
    auto original = p;
    int length = last - first;
    for (int i = 0; i < length; ++i)
        p.at(first + i) = original.at(first + (i + middle - first) % length);
}

uint32_t rank_permutation(const Permutation& p, int n, const std::vector<uint32_t>& factorial) {
    uint32_t rank = 0;
    uint32_t available = (1u << n) - 1;
    for (int i = 0; i < n; ++i) {
        rank += std::popcount(available & ((1u << p.at(i)) - 1)) * factorial.at(n - i - 1);
        available &= ~(1u << p.at(i));
    }
    return rank;
}

Permutation unrank_permutation(uint32_t rank, int n, const std::vector<uint32_t>& factorial) {
    Permutation p{};
    uint32_t available = (1u << n) - 1;
    for (int i = 0; i < n; ++i) {
        uint32_t selection = rank / factorial.at(n - i - 1);
        rank %= factorial.at(n - i - 1);
        uint32_t choices = available;
        while (selection--) choices &= choices - 1;
        p.at(i) = std::countr_zero(choices);
        available &= ~(1u << p.at(i));
    }
    return p;
}

int main(int argc, char** argv) {
    try {
        if (argc < 3 || argc > 4) throw std::runtime_error("usage: campaign_execution_tables N OUTPUT.json [tail]");
        bool prefer_tail = argc == 4 && std::string(argv[3]) == "tail";
        if (argc == 4 && !prefer_tail) throw std::runtime_error("unknown tie-breaking mode");
        int n = std::stoi(argv[1]);
        if (n < 1 || n > 8) throw std::runtime_error("N must be between 1 and 8");
        auto started = std::chrono::steady_clock::now();
        std::vector<uint32_t> factorial(n + 1, 1);
        for (int i = 1; i <= n; ++i) factorial[i] = factorial[i - 1] * i;
        std::vector<std::pair<int, int>> cuts;
        std::array<std::array<uint32_t, 9>, 9> cut_id{};
        for (int a = 0; a <= n; ++a)
            for (int d = 0; d <= n - a; ++d) {
                cut_id.at(a).at(d) = cuts.size();
                cuts.emplace_back(a, d);
            }
        uint32_t cut_count = cuts.size();
        uint32_t state_count = factorial[n] * cut_count;
        std::vector<uint16_t> distance(state_count, std::numeric_limits<uint16_t>::max());
        std::vector<uint32_t> queue;
        queue.reserve(state_count);
        uint32_t goal = (factorial[n] - 1) * cut_count + cut_id.at(n).at(0);
        distance.at(goal) = 0;
        queue.push_back(goal);
        auto neighbors = [&](uint32_t id, auto visit) {
            auto [a, d] = cuts.at(id % cut_count);
            int b = n - a - d;
            auto p = unrank_permutation(id / cut_count, n, factorial);
            auto emit = [&](const Permutation& q, int next_a, int next_d, int code) {
                visit(rank_permutation(q, n, factorial) * cut_count + cut_id.at(next_a).at(next_d), code);
            };
            if (a) {
                auto q = p;
                rotate_segment(q, 0, 1, a);
                emit(q, a - 1, d + 1, 0);
            }
            if (d) {
                auto q = p;
                rotate_segment(q, 0, a, a + 1);
                emit(q, a + 1, d - 1, 1);
                q = p;
                rotate_segment(q, a, a + 1, a + d);
                emit(q, a, d - 1, 2);
            }
            if (b) {
                auto q = p;
                rotate_segment(q, a, a + d, a + d + 1);
                emit(q, a, d + 1, 3);
            }
        };
        for (size_t head = 0; head < queue.size(); ++head) {
            auto id = queue.at(head);
            neighbors(id, [&](uint32_t next, int) {
                if (distance.at(next) == std::numeric_limits<uint16_t>::max()) {
                    distance.at(next) = distance.at(id) + 1;
                    queue.push_back(next);
                }
            });
        }
        if (queue.size() != state_count) throw std::runtime_error("unreachable states");
        std::vector<uint8_t> terminal_run(state_count, 0);
        if (prefer_tail) {
            for (auto id : queue) {
                neighbors(id, [&](uint32_t next, int code) {
                    if (distance.at(next) + 1 == distance.at(id)) {
                        auto run = terminal_run.at(next);
                        if (code == 1 && run == distance.at(next)) ++run;
                        if (run > terminal_run.at(id)) terminal_run.at(id) = run;
                    }
                });
            }
        }
        std::ofstream out(argv[2]);
        if (!out) throw std::runtime_error("cannot open output file");
        out << "{\n  \"n\": " << n << ",\n  \"endpoint\": \"A reversed sorted; D and B empty\",\n"
            << "  \"initial_order\": \"Lexicographic permutations in D, normalized to sorted target ranks\",\n"
            << "  \"move_codes\": [\"AD\",\"DA\",\"DB\",\"BD\"],\n"
            << "  \"encoding\": \"Strings of single-digit move codes in execution order\",\n"
            << "  \"tie_breaking\": \"" << (prefer_tail ? "maximum terminal parking run" : "first shortest predecessor") << "\",\n"
            << "  \"state_count\": " << state_count << ",\n  \"plans\": [";
        std::map<uint16_t, uint32_t> histogram;
        for (uint32_t rank = 0; rank < factorial[n]; ++rank) {
            uint32_t current = rank * cut_count + cut_id.at(0).at(n);
            ++histogram[distance.at(current)];
            std::string word;
            while (distance.at(current)) {
                bool found = false;
                uint32_t selected = current;
                neighbors(current, [&](uint32_t next, int code) {
                    if (!found && distance.at(next) + 1 == distance.at(current)) {
                        auto run = terminal_run.at(next);
                        if (code == 1 && run == distance.at(next)) ++run;
                        if (prefer_tail && run != terminal_run.at(current)) return;
                        found = true;
                        selected = next;
                        word += char('0' + code);
                    }
                });
                if (!found) throw std::runtime_error("no shortest predecessor");
                current = selected;
            }
            out << (rank ? "," : "") << '"' << word << '"';
        }
        double seconds = std::chrono::duration<double>(std::chrono::steady_clock::now() - started).count();
        out << "],\n  \"elapsed_seconds\": " << seconds << ",\n  \"distance_histogram\": {";
        bool first = true;
        for (auto [cost, count] : histogram) {
            out << (first ? "" : ",") << '"' << cost << "\":" << count;
            first = false;
        }
        out << "}\n}\n";
        std::cerr << "n=" << n << " states=" << state_count << " max=" << histogram.rbegin()->first
                  << " seconds=" << seconds << '\n';
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
