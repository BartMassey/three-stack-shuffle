#include <algorithm>
#include <array>
#include <bit>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <map>
#include <numeric>
#include <stdexcept>
#include <string>
#include <vector>

using Permutation = std::array<uint8_t, 12>;

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
        rank += std::popcount(available & ((1u << p[i]) - 1)) * factorial[n - i - 1];
        available &= ~(1u << p[i]);
    }
    return rank;
}

Permutation unrank_permutation(uint32_t rank, int n, const std::vector<uint32_t>& factorial) {
    Permutation p{};
    uint32_t available = (1u << n) - 1;
    for (int i = 0; i < n; ++i) {
        uint32_t selection = rank / factorial[n - i - 1];
        rank %= factorial[n - i - 1];
        uint32_t choices = available;
        while (selection--) choices &= choices - 1;
        p[i] = std::countr_zero(choices);
        available &= ~(1u << p[i]);
    }
    return p;
}

int main(int argc, char** argv) {
    try {
        if (argc < 2 || argc > 4) throw std::runtime_error("usage: exact_search N [OUTPUT.json] [PLANS.json]");
        int n = std::stoi(argv[1]);
        if (n < 1 || n > 10) throw std::runtime_error("N must be between 1 and 10");
        auto start = std::chrono::steady_clock::now();
        std::vector<uint32_t> factorial(n + 1, 1);
        for (int i = 1; i <= n; ++i) factorial[i] = factorial[i - 1] * i;
        std::vector<std::pair<int,int>> cuts;
        int cut_id[12][12]{};
        for (int a = 0; a <= n; ++a) {
            for (int d = 0; d <= n - a; ++d) {
                cut_id[a][d] = cuts.size();
                cuts.emplace_back(a, d);
            }
        }
        uint32_t cut_count = cuts.size();
        uint64_t state_count = uint64_t(factorial[n]) * cut_count;
        if (state_count > std::numeric_limits<uint32_t>::max()) throw std::runtime_error("state IDs exceed uint32_t");
        std::vector<uint16_t> distance(state_count, std::numeric_limits<uint16_t>::max());
        std::vector<uint32_t> queue;
        queue.reserve(state_count);
        uint32_t initial = cut_id[0][n];
        distance[initial] = 0;
        queue.push_back(initial);
        auto visit = [&](const Permutation& p, int a, int d, uint16_t next_distance) {
            uint32_t id = rank_permutation(p, n, factorial) * cut_count + cut_id[a][d];
            if (distance[id] == std::numeric_limits<uint16_t>::max()) {
                distance[id] = next_distance;
                queue.push_back(id);
            }
        };
        for (size_t head = 0; head < queue.size(); ++head) {
            uint32_t id = queue[head];
            auto [a, d] = cuts[id % cut_count];
            int b = n - a - d;
            auto p = unrank_permutation(id / cut_count, n, factorial);
            uint16_t next_distance = distance[id] + 1;
            if (a) {
                auto q = p;
                rotate_segment(q, 0, 1, a);
                visit(q, a - 1, d + 1, next_distance);
            }
            if (d) {
                auto q = p;
                rotate_segment(q, 0, a, a + 1);
                visit(q, a + 1, d - 1, next_distance);
                q = p;
                rotate_segment(q, a, a + 1, a + d);
                visit(q, a, d - 1, next_distance);
            }
            if (b) {
                auto q = p;
                rotate_segment(q, a, a + d, a + d + 1);
                visit(q, a, d + 1, next_distance);
            }
        }
        if (queue.size() != state_count) throw std::runtime_error("unreachable states");
        std::map<uint16_t, uint32_t> histogram;
        uint64_t sum = 0;
        std::vector<uint16_t> target_distances;
        for (uint32_t rank = 0; rank < factorial[n]; ++rank) {
            auto value = distance[rank * cut_count + initial];
            if (value % 2) throw std::runtime_error("odd distance to central-stack target");
            ++histogram[value];
            sum += value;
            target_distances.push_back(value);
        }
        uint16_t reversal = target_distances.back();
        if (reversal != 4 * (n - 1)) throw std::runtime_error("reversal differs from expected 4(N-1)");
        auto seconds = std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count();
        std::ofstream file;
        if (argc >= 3) file.open(argv[2]);
        if (argc >= 3 && !file) throw std::runtime_error("cannot open output file");
        std::ostream& out = argc >= 3 ? file : std::cout;
        out.precision(15);
        out << "{\n  \"n\": " << n << ",\n  \"model\": \"Three stacks A-D-B; only adjacent moves; move one top card to neighbor top; top-to-bottom sequences\",\n";
        out << "  \"initial_D\": [";
        for (int i = 0; i < n; ++i) out << (i ? ", " : "") << i;
        out << "],\n  \"state_count\": " << state_count << ",\n  \"visited_state_count\": " << queue.size();
        out << ",\n  \"target_count\": " << factorial[n] << ",\n  \"distance_sum\": " << sum;
        out << ",\n  \"mean_numerator\": " << sum << ",\n  \"mean_denominator\": " << factorial[n];
        out << ",\n  \"mean\": " << double(sum) / factorial[n];
        out << ",\n  \"maximum\": " << histogram.rbegin()->first << ",\n  \"reversal_distance\": " << reversal;
        out << ",\n  \"elapsed_seconds\": " << seconds << ",\n  \"distance_histogram\": {";
        bool first = true;
        for (auto [value, count] : histogram) {
            out << (first ? "\n" : ",\n") << "    \"" << value << "\": " << count;
            first = false;
        }
        out << "\n  },\n  \"target_distance_order\": \"Lexicographic permutations of 0..N-1, top to bottom in D\",\n  \"target_distances\": [";
        for (size_t i = 0; i < target_distances.size(); ++i) out << (i ? "," : "") << target_distances[i];
        out << "]\n}\n";
        if (argc == 4) {
            if (n > 8) throw std::runtime_error("compact plans require N <= 8");
            std::ofstream plans(argv[3]);
            if (!plans) throw std::runtime_error("cannot open plans output file");
            plans << "{\n  \"n\": " << n
                  << ",\n  \"move_codes\": [\"AD\",\"DA\",\"DB\",\"BD\"],\n"
                  << "  \"encoding\": \"Start integer at 1; append moves in execution order by shifting left 2 and OR move code; decode low bits then reverse\",\n"
                  << "  \"target_order\": \"Lexicographic permutations of 0..N-1, top to bottom in D\",\n"
                  << "  \"plans\": [";
            for (uint32_t rank = 0; rank < factorial[n]; ++rank) {
                uint32_t current = rank * cut_count + initial;
                std::vector<uint8_t> reverse_plan;
                while (distance[current]) {
                    auto [a, d] = cuts[current % cut_count];
                    int b = n - a - d;
                    auto p = unrank_permutation(current / cut_count, n, factorial);
                    bool found = false;
                    auto predecessor = [&](const Permutation& q, int next_a, int next_d, uint8_t code) {
                        if (found) return;
                        uint32_t next = rank_permutation(q, n, factorial) * cut_count + cut_id[next_a][next_d];
                        if (distance[next] + 1 == distance[current]) {
                            current = next;
                            reverse_plan.push_back(code ^ 1);
                            found = true;
                        }
                    };
                    if (a) {
                        auto q = p;
                        rotate_segment(q, 0, 1, a);
                        predecessor(q, a - 1, d + 1, 0);
                    }
                    if (d) {
                        auto q = p;
                        rotate_segment(q, 0, a, a + 1);
                        predecessor(q, a + 1, d - 1, 1);
                        q = p;
                        rotate_segment(q, a, a + 1, a + d);
                        predecessor(q, a, d - 1, 2);
                    }
                    if (b) {
                        auto q = p;
                        rotate_segment(q, a, a + d, a + d + 1);
                        predecessor(q, a, d + 1, 3);
                    }
                    if (!found) throw std::runtime_error("no shortest-path predecessor");
                }
                uint64_t packed = 1;
                for (auto move = reverse_plan.rbegin(); move != reverse_plan.rend(); ++move)
                    packed = (packed << 2) | *move;
                plans << (rank ? "," : "") << packed;
            }
            plans << "]\n}\n";
        }
        std::cerr << "n=" << n << " states=" << state_count << " mean=" << double(sum) / factorial[n]
                  << " max=" << histogram.rbegin()->first << " reversal=" << reversal << " seconds=" << seconds << '\n';
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
