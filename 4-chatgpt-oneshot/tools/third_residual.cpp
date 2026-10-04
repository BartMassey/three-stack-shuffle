#define main campaign_search_main
#include "campaign_exact.cpp"
#undef main
#include <fstream>

int main(int argc, char** argv) {
    try {
        if (argc != 4 || (std::string(argv[1]) != "dump"
                         && std::string(argv[1]) != "partition"
                         && std::string(argv[1]) != "selected"))
            throw std::runtime_error("usage: third_residual dump|partition|selected N OUTPUT");
        int n = std::stoi(argv[2]);
        if (n < 1 || n > 6) throw std::runtime_error("unsupported size");
        Database database(n);
        Database pdb(std::min(4, n));
        Search search(n, pdb);
        bool partition = std::string(argv[1]) != "dump";
        if (partition) {
            build(pdb);
            search.use_residual_structural = true;
            search.use_residual_partition = std::string(argv[1]) == "partition";
            search.use_selected_residual_partition = std::string(argv[1]) == "selected";
            for (uint32_t mask = 0; mask < (1u << n); ++mask) {
                if (std::popcount(mask) != pdb.n) continue;
                Pattern pattern{mask, {}};
                int label = 0;
                for (int card = 0; card < n; ++card)
                    if (mask & (1u << card)) pattern.labels.at(card) = label++;
                search.patterns.push_back(pattern);
            }
        }
        std::ofstream output(argv[3], std::ios::binary);
        uint64_t count = database.factorial.at(n) * database.cuts.size();
        for (uint64_t id = 0; id < count; ++id) {
            auto state = database.state(id);
            output.put(partition ? search.heuristic(state) : residual_bound(state, n));
        }
        return output.good() ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
